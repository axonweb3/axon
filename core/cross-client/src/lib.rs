#![allow(dead_code, unused_variables, clippy::derive_partial_eq_without_eq)]

mod adapter;
pub mod crosschain_abi;
mod error;
mod generated;
mod monitor;
mod sidechain;
mod task;

pub use adapter::{CrossChainDBImpl, DefaultCrossChainAdapter};
pub use task::message::{
    CrosschainMessageHandler, END_GOSSIP_BUILD_CKB_TX, END_GOSSIP_CKB_TX_SIGNATURE,
};

use std::sync::Arc;

use arc_swap::ArcSwap;
use ckb_types::{core::TransactionView, prelude::*};
use ethers_contract::decode_logs;
use ethers_core::abi::{self, AbiEncode, AbiType, Detokenize, RawLog};

use common_config_parser::types::ConfigCrossChain;
use common_crypto::{
    PrivateKey, Secp256k1RecoverablePrivateKey, Signature, ToPublicKey, UncompressedPublicKey,
};
use protocol::tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use protocol::traits::{CkbClient, Context, CrossAdapter, Crosschain};
use protocol::types::{
    Address, Block, BlockNumber, Direction, Eip1559Transaction, Hash, Hasher, Log, Proof,
    RequestTxHashes, Requests, SignedTransaction, TransactionAction, Transfer, UnsignedTransaction,
    UnverifiedTransaction, H160, H256, MAX_BLOCK_GAS_LIMIT, MAX_PRIORITY_FEE_PER_GAS, U256,
};
use protocol::{async_trait, lazy::CHAIN_ID, tokio, ProtocolResult};

use core_executor::CROSSCHAIN_CONTRACT_ADDRESS;

use crate::error::CrossChainError;
use crate::task::{message::CrosschainMessage, RequestCkbTask};
use crate::{adapter::fixed_array, monitor::CrossChainMonitor, sidechain::SidechainTask};

pub const CKB_BLOCK_INTERVAL: u64 = 8; // second
pub const NON_FORK_BLOCK_GAP: u64 = 24;

lazy_static::lazy_static! {
    pub static ref CKB_TIP: ArcSwap<u64> = ArcSwap::from_pointee(0);
}

pub struct CrossChainImpl<Adapter> {
    priv_key: Secp256k1RecoverablePrivateKey,
    address:  H160,
    log_rx:   UnboundedReceiver<(Vec<Vec<Log>>, H256)>,
    req_rx:   UnboundedReceiver<Vec<TransactionView>>,
    reqs_tx:  UnboundedSender<Requests>,

    adapter: Arc<Adapter>,
}

impl<Adapter: CrossAdapter + 'static> CrossChainImpl<Adapter> {
    pub async fn new<C: CkbClient + 'static>(
        private_key: &[u8],
        config: ConfigCrossChain,
        ckb_client: Arc<C>,
        adapter: Arc<Adapter>,
    ) -> (
        Self,
        CrosschainHandler<C>,
        UnboundedSender<CrosschainMessage>,
    ) {
        let priv_key = Secp256k1RecoverablePrivateKey::try_from(private_key)
            .expect("Invalid secp private key");
        let address = Address::from_pubkey_bytes(priv_key.pub_key().to_uncompressed_bytes())
            .unwrap()
            .0;

        let (log_tx, log_rx) = unbounded_channel();
        let (req_tx, req_rx) = unbounded_channel();
        let client_clone = Arc::clone(&ckb_client);
        let init_monitor_number = adapter
            .get_monitor_ckb_number(Context::new())
            .await
            .unwrap_or(config.start_block_number);

        let (reqs_tx, reqs_rx) = unbounded_channel();
        let (ckb_task, crosschain_net_handler) =
            RequestCkbTask::new(address, private_key, reqs_rx, Arc::clone(&adapter)).await;

        tokio::spawn(async move { ckb_task.run().await });

        let adapter_clone = Arc::clone(&adapter);
        tokio::spawn(async move {
            CrossChainMonitor::new(
                client_clone,
                req_tx,
                config.start_block_number,
                config.acs_lock_code_hash,
                config.request_type_code_hash,
                adapter_clone,
            )
            .await
            .run()
            .await
        });

        let handler = CrosschainHandler::new(priv_key.clone(), log_tx, config, ckb_client);
        let crosschain = CrossChainImpl {
            priv_key,
            address,
            log_rx,
            req_rx,
            reqs_tx,
            adapter,
        };

        crosschain
            .recover_tasks()
            .await
            .expect("Recover crosschain tasks");

        (crosschain, handler, crosschain_net_handler)
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some((logs, block_hash)) = self.log_rx.recv() => {
                    let (to_ckbs, to_ckb_alerts) = self.classify_logs(logs, block_hash).await.unwrap();

                    if !to_ckbs.is_empty() {
                        let adapter_clone = Arc::clone(&self.adapter);
                        tokio::spawn(async move {
                            match build_ckb_txs(to_ckbs).await {
                                Ok((reqs, stx)) => {
                                    let ctx = Context::new();
                                    adapter_clone.insert_in_process(
                                        ctx.clone(),
                                        &rlp::encode(&reqs).freeze(),
                                        stx.pack().as_slice()
                                    )
                                    .await
                                    .unwrap();
                                    let _ = adapter_clone.send_ckb_tx(ctx, stx.into()).await;
                                },

                                Err(e) => log::error!("[crosschain]: crosschain error {:?}", e),
                            };

                        });
                    }
                }

                Some(reqs) = self.req_rx.recv() => {
                    log::info!("[cross-chain]: receive requests {:?} from CKB", reqs);
                    let (reqs, stx) = self.build_axon_txs(reqs).await;
                    self.adapter.insert_in_process(
                        Context::new(),
                        &rlp::encode(&reqs).freeze(),
                        &rlp::encode(&stx).freeze()
                    )
                    .await
                    .unwrap();
                    self.adapter.send_axon_tx(Context::new(), stx).await.unwrap();
                }
            }
        }
    }

    async fn recover_tasks(&self) -> ProtocolResult<()> {
        let ctx = Context::new();

        for item in self.adapter.get_all_in_process(ctx.clone()).await?.iter() {
            let req: Requests =
                rlp::decode(&item.0).map_err(|e| CrossChainError::Adapter(e.to_string()))?;
            let direction = req.direction();

            if direction.is_from_ckb() {
                let _ = self
                    .adapter
                    .send_axon_tx(
                        ctx.clone(),
                        rlp::decode(&item.1)
                            .map_err(|e| CrossChainError::Adapter(e.to_string()))?,
                    )
                    .await;
            }
        }

        Ok(())
    }

    async fn build_axon_txs(&self, txs: Vec<TransactionView>) -> (Requests, SignedTransaction) {
        let reqs = txs
            .iter()
            .map(|tx| {
                let type_script = tx.output(1).unwrap().type_().to_opt().unwrap();
                let request_args = generated::Transfer::new_unchecked(type_script.args().unpack());

                Transfer {
                    direction:     Direction::FromCkb,
                    tx_hash:       H256::from_slice(&tx.hash().raw_data()),
                    address:       H160::from_slice(&request_args.axon_address().raw_data()),
                    erc20_address: H160::from_slice(&request_args.e_r_c20_address().raw_data()),
                    ckb_amount:    u64::from_le_bytes(fixed_array(
                        &request_args.ckb_amount().raw_data(),
                    )),
                    sudt_amount:   u128::from_le_bytes(fixed_array(
                        &request_args.s_u_d_t_amount().raw_data(),
                    )),
                }
            })
            .collect::<Vec<_>>();

        let call_data = crosschain_abi::CrossFromCKBCall {
            records: reqs
                .iter()
                .map(|req| crosschain_abi::CkbtoAxonRecord {
                    to:            req.address,
                    token_address: req.erc20_address,
                    s_udt_amount:  req.sudt_amount.into(),
                    ckb_amount:    req.ckb_amount.into(),
                    tx_hash:       req.tx_hash.0,
                })
                .collect(),
            nonce:   U256::zero(),
        };

        let tx = UnsignedTransaction::Eip1559(Eip1559Transaction {
            nonce:                    self
                .adapter
                .nonce(Context::new(), self.address)
                .await
                .unwrap()
                + 1,
            max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS.into(),
            gas_price:                U256::zero(),
            gas_limit:                MAX_BLOCK_GAS_LIMIT.into(),
            action:                   TransactionAction::Call(CROSSCHAIN_CONTRACT_ADDRESS),
            value:                    U256::zero(),
            data:                     AbiEncode::encode(call_data).into(),
            access_list:              vec![],
        });

        let id = **CHAIN_ID.load();
        let signature = self.priv_key.sign_message(
            &Hasher::digest(tx.encode(id, None))
                .as_bytes()
                .try_into()
                .unwrap(),
        );

        let utx = UnverifiedTransaction {
            unsigned:  tx,
            signature: Some(signature.to_bytes().into()),
            chain_id:  id,
            hash:      Default::default(),
        }
        .calc_hash();

        let stx: SignedTransaction = utx.try_into().unwrap();

        log::info!("[cross-chain]: build cross from CKB transaction {:?}", stx);

        (Requests(reqs), stx)
    }

    async fn classify_logs(
        &self,
        logs: Vec<Vec<Log>>,
        block_hash: H256,
    ) -> ProtocolResult<(
        Vec<crosschain_abi::CrossToCKBFilter>,
        Vec<crosschain_abi::CrossToCKBAlertFilter>,
    )> {
        let logs = logs
            .iter()
            .map(|inner_logs| {
                inner_logs
                    .iter()
                    .map(|log| RawLog {
                        topics: log.topics.clone(),
                        data:   log.data.clone(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut to_ckbs = Vec::new();
        let mut to_ckb_alerts = Vec::new();

        for log in logs.iter() {
            if log.is_empty() {
                continue;
            }

            let last_log = log.last().cloned().unwrap();

            if let Ok(event) =
                decode_logs::<crosschain_abi::CrossFromCKBFilter>(&[last_log.clone()])
            {
                log::info!(
                    "[crosschain]: Complete cross from CKB, request count {:?}, axon block hash {:?}",
                    event[0].records.len(), block_hash
                );

                let _ = self
                    .adapter
                    .remove_in_process(
                        Context::new(),
                        &rlp::encode::<Requests>(&(event[0].clone().into())),
                    )
                    .await;

                let hashes = event[0]
                    .records
                    .iter()
                    .map(|r| H256(r.4))
                    .collect::<Vec<_>>();
                self.adapter
                    .insert_record(
                        Context::new(),
                        RequestTxHashes::new_from_ckb(hashes),
                        block_hash,
                    )
                    .await?;
            } else if let Ok(event) =
                decode_logs::<crosschain_abi::CrossToCKBFilter>(&[last_log.clone()])
            {
                to_ckbs.push(event[0].clone());
            } else if let Ok(event) =
                decode_logs::<crosschain_abi::CrossToCKBAlertFilter>(&[last_log])
            {
                to_ckb_alerts.push(event[0].clone());
            }
        }

        Ok((to_ckbs, to_ckb_alerts))
    }
}

pub struct CrosschainHandler<C> {
    priv_key:   Secp256k1RecoverablePrivateKey,
    logs_tx:    UnboundedSender<(Vec<Vec<Log>>, H256)>,
    config:     ConfigCrossChain,
    ckb_client: Arc<C>,
}

#[async_trait]
impl<C: CkbClient + 'static> Crosschain for CrosschainHandler<C> {
    async fn set_evm_log(
        &self,
        ctx: Context,
        block_number: BlockNumber,
        block_hash: Hash,
        logs: &[Vec<Log>],
    ) {
        if let Err(e) = self.logs_tx.send((logs.to_vec(), block_hash)) {
            log::error!("[cross-chain]: send log to process error {:?}", e);
        }
    }

    async fn set_checkpoint(&self, ctx: Context, block: Block, proof: Proof) {
        let priv_key = self.priv_key.clone();
        let node_address = self.config.node_address;
        let admin_address = self.config.admin_address;
        let selection_lock_hash = self.config.selection_lock_hash;
        let checkpoint_type_hash = self.config.checkpoint_type_hash;
        let ckb_client = Arc::clone(&self.ckb_client);

        tokio::spawn(async move {
            SidechainTask::new(
                priv_key,
                node_address,
                admin_address,
                selection_lock_hash,
                checkpoint_type_hash,
            )
            .run(Arc::clone(&ckb_client), block, proof)
            .await
        });
    }
}

impl<C: CkbClient + 'static> CrosschainHandler<C> {
    fn new(
        priv_key: Secp256k1RecoverablePrivateKey,
        logs_tx: UnboundedSender<(Vec<Vec<Log>>, H256)>,
        config: ConfigCrossChain,
        ckb_client: Arc<C>,
    ) -> Self {
        CrosschainHandler {
            priv_key,
            logs_tx,
            config,
            ckb_client,
        }
    }
}

pub fn decode_resp_nonce(data: &[u8]) -> U256 {
    U256::from_tokens(abi::decode(&[U256::param_type()], data).unwrap()).unwrap()
}

async fn build_ckb_txs(
    events: Vec<crosschain_abi::CrossToCKBFilter>,
) -> ProtocolResult<(Requests, TransactionView)> {
    todo!()
}

impl From<crosschain_abi::CrossFromCKBFilter> for Requests {
    fn from(logs: crosschain_abi::CrossFromCKBFilter) -> Self {
        Requests(
            logs.records
                .into_iter()
                .map(|r| Transfer {
                    direction:     Direction::FromCkb,
                    address:       r.0,
                    erc20_address: r.1,
                    sudt_amount:   r.2.as_u128(),
                    ckb_amount:    r.3.as_u64(),
                    tx_hash:       H256(r.4),
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_data() {
        let transfer = generated::TransferBuilder::default().build();
        assert_eq!(
            transfer.axon_address().raw_data(),
            H160::default().0.to_vec()
        );

        let byte32 = ckb_types::packed::Byte32Builder::default().build();
        assert_eq!(byte32.raw_data(), H256::default().0.to_vec());
    }

    #[test]
    #[ignore]
    fn gen_abi_binding() {
        ethers_contract::Abigen::new("crosschain", "./crosschain_abi.json")
            .unwrap()
            .generate()
            .unwrap()
            .write_to_file("src/crosschain_abi.rs")
            .unwrap();
    }
}
