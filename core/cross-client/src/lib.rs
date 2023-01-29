#![allow(
    dead_code,
    unused_variables,
    clippy::derive_partial_eq_without_eq,
    clippy::uninlined_format_args
)]

mod abi;
mod adapter;
mod error;
mod generated;
pub mod monitor;
mod sidechain;
mod task;

pub use abi::{crosschain_abi, wckb_abi};
pub use adapter::{CrossChainDBImpl, DefaultCrossChainAdapter};
use protocol::codec::ProtocolCodec;
pub use task::message::{
    CrossChainMessageHandler, END_GOSSIP_BUILD_CKB_TX, END_GOSSIP_CKB_TX_SIGNATURE,
};

use std::sync::Arc;

use arc_swap::ArcSwap;
use ckb_sdk::constants::ONE_CKB;
use ckb_types::{core::TransactionView, prelude::*};
use ethers_contract::decode_logs;
use ethers_core::abi::{decode as abi_decode, AbiEncode, AbiType, Detokenize, RawLog};

use common_config_parser::types::ConfigCrossChain;
use common_crypto::{
    PrivateKey, Secp256k1RecoverablePrivateKey, Signature, ToPublicKey, UncompressedPublicKey,
};
use protocol::tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use protocol::traits::{CkbClient, Context, CrossAdapter, CrossChain};
use protocol::types::{
    Address, Block, BlockNumber, Direction, Eip1559Transaction, Hash, Hasher, Log, Proof,
    RequestTxHashes, Requests, SignedTransaction, TransactionAction, Transfer, UnsignedTransaction,
    UnverifiedTransaction, H160, H256, MAX_BLOCK_GAS_LIMIT, MAX_PRIORITY_FEE_PER_GAS, U256,
};
use protocol::{async_trait, lazy::CHAIN_ID, tokio, ProtocolResult};

use crate::error::CrossChainError;
use crate::task::{message::CrossChainMessage, RequestCkbTask};
use crate::{adapter::fixed_array, monitor::CrossChainMonitor, sidechain::SidechainTask};

pub const CKB_BLOCK_INTERVAL: u64 = 8; // second
pub const NON_FORK_BLOCK_GAP: u64 = 24;
const BASE_CROSSCHAIN_CELL_CAPACITY: u64 = 200 * ONE_CKB;

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

    cross_chain_address: H160,
    wckb_address:        H160,
}

impl<Adapter: CrossAdapter + 'static> CrossChainImpl<Adapter> {
    pub async fn new<C: CkbClient + 'static>(
        private_key: &[u8],
        config: ConfigCrossChain,
        ckb_client: Arc<C>,
        adapter: Arc<Adapter>,
        cross_chain_address: H160,
        wckb_address: H160,
    ) -> (
        Self,
        CrossChainHandler<C>,
        UnboundedSender<CrossChainMessage>,
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

        let handler = CrossChainHandler::new(priv_key.clone(), log_tx, config, ckb_client);
        let crosschain = CrossChainImpl {
            priv_key,
            address,
            log_rx,
            req_rx,
            reqs_tx,
            adapter,
            cross_chain_address,
            wckb_address,
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
                        let tx_clone = self.reqs_tx.clone();
                        tokio::spawn(async move {
                            build_ckb_tx_process(to_ckbs, tx_clone, &self.wckb_address).await;
                        });
                    }

                    if !to_ckb_alerts.is_empty() {
                        log::error!("to_ckb_alerts");
                    }
                }

                Some(reqs) = self.req_rx.recv() => {
                    log::info!("[cross-chain]: receive requests {:?} from CKB", reqs);
                    let nonce = self.nonce(self.address).await;
                    let (reqs, stx) = build_axon_txs(reqs, nonce, &self.priv_key, self.cross_chain_address);
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

    async fn classify_logs(
        &self,
        logs: Vec<Vec<Log>>,
        block_hash: H256,
    ) -> ProtocolResult<(
        Vec<crosschain_abi::CrossToCKBFilter>,
        Vec<crosschain_abi::CrossToCKBAlertFilter>,
    )> {
        let logs = logs.iter().filter_map(|inner_logs| {
            inner_logs.last().cloned().map(|l| RawLog {
                topics: l.topics.clone(),
                data:   l.data,
            })
        });

        let mut to_ckbs = Vec::new();
        let mut to_ckb_alerts = Vec::new();

        for log in logs {
            if let Ok(event) = decode_logs::<crosschain_abi::CrossFromCKBFilter>(&[log.clone()]) {
                let key = rlp::encode::<Requests>(&(event[0].clone().into()));
                let relay_tx = SignedTransaction::decode(
                    self.adapter
                        .get_in_process(Context::new(), &key)
                        .await?
                        .unwrap(),
                )?;
                self.adapter.remove_in_process(Context::new(), &key).await?;

                let hashes = event[0]
                    .records
                    .iter()
                    .map(|r| H256(r.tx_hash))
                    .collect::<Vec<_>>();
                self.adapter
                    .insert_record(
                        Context::new(),
                        RequestTxHashes::new_from_ckb(hashes),
                        relay_tx.transaction.hash,
                    )
                    .await?;

                log::info!(
                    "[crosschain]: Complete cross from CKB, request count {:?}, axon tx hash {:?}",
                    event[0].records.len(),
                    relay_tx.transaction.hash
                );
            } else if let Ok(event) =
                decode_logs::<crosschain_abi::CrossToCKBFilter>(&[log.clone()])
            {
                log::info!(
                    "[crosschain]: Complete cross from Axon, axon block hash {:?}",
                    block_hash
                );
                to_ckbs.push(event[0].clone());
            } else if let Ok(event) = decode_logs::<crosschain_abi::CrossToCKBAlertFilter>(&[log]) {
                to_ckb_alerts.push(event[0].clone());
            }
        }

        Ok((to_ckbs, to_ckb_alerts))
    }

    async fn nonce(&self, address: H160) -> U256 {
        self.adapter
            .nonce(Context::new(), self.address)
            .await
            .unwrap()
            + 1
    }
}

pub struct CrossChainHandler<C> {
    priv_key:   Secp256k1RecoverablePrivateKey,
    logs_tx:    UnboundedSender<(Vec<Vec<Log>>, H256)>,
    config:     ConfigCrossChain,
    ckb_client: Arc<C>,
}

#[async_trait]
impl<C: CkbClient + 'static> CrossChain for CrossChainHandler<C> {
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

impl<C: CkbClient + 'static> CrossChainHandler<C> {
    fn new(
        priv_key: Secp256k1RecoverablePrivateKey,
        logs_tx: UnboundedSender<(Vec<Vec<Log>>, H256)>,
        config: ConfigCrossChain,
        ckb_client: Arc<C>,
    ) -> Self {
        CrossChainHandler {
            priv_key,
            logs_tx,
            config,
            ckb_client,
        }
    }
}

pub fn decode_resp_nonce(data: &[u8]) -> U256 {
    U256::from_tokens(abi_decode(&[U256::param_type()], data).unwrap()).unwrap()
}

async fn build_ckb_tx_process(
    to_ckbs: Vec<crosschain_abi::CrossToCKBFilter>,
    request_tx: UnboundedSender<Requests>,
    wckb_address: &H160,
) {
    let _ = request_tx.send(Requests(
        to_ckbs
            .iter()
            .map(|log| {
                let (s_amount, c_amount) = if &log.token == wckb_address {
                    (0, log.amount.as_u64() + BASE_CROSSCHAIN_CELL_CAPACITY)
                } else {
                    (log.amount.as_u128(), BASE_CROSSCHAIN_CELL_CAPACITY)
                };

                Transfer {
                    direction:     Direction::FromAxon,
                    ckb_address:   log.to.clone(),
                    address:       H160::default(),
                    erc20_address: log.token,
                    sudt_amount:   s_amount,
                    ckb_amount:    c_amount,
                    tx_hash:       H256::default(),
                }
            })
            .collect::<Vec<_>>(),
    ));
}

pub fn build_axon_txs(
    txs: Vec<TransactionView>,
    addr_nonce: U256,
    priv_key: &Secp256k1RecoverablePrivateKey,
    cross_chain_address: H160,
) -> (Requests, SignedTransaction) {
    let reqs = txs
        .iter()
        .map(|tx| {
            let type_script = tx.output(1).unwrap().type_().to_opt().unwrap();
            let request_args = generated::Transfer::new_unchecked(type_script.args().unpack());

            Transfer {
                direction:     Direction::FromCkb,
                ckb_address:   String::new(),
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
        nonce:                    addr_nonce,
        max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS.into(),
        gas_price:                U256::zero(),
        gas_limit:                MAX_BLOCK_GAS_LIMIT.into(),
        action:                   TransactionAction::Call(cross_chain_address),
        value:                    U256::zero(),
        data:                     AbiEncode::encode(call_data).into(),
        access_list:              vec![],
    });

    let id = **CHAIN_ID.load();
    let signature = priv_key.sign_message(
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

impl From<crosschain_abi::CrossFromCKBFilter> for Requests {
    fn from(logs: crosschain_abi::CrossFromCKBFilter) -> Self {
        Requests(
            logs.records
                .into_iter()
                .map(|r| Transfer {
                    direction:     Direction::FromCkb,
                    ckb_address:   String::new(),
                    address:       r.to,
                    erc20_address: r.token_address,
                    sudt_amount:   r.s_udt_amount.as_u128(),
                    ckb_amount:    r.ckb_amount.as_u64(),
                    tx_hash:       H256(r.tx_hash),
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
        ethers_contract::Abigen::new("wckb", "./wckb_abi.json")
            .unwrap()
            .generate()
            .unwrap()
            .write_to_file("src/abi/wckb_abi.rs")
            .unwrap();
    }
}
