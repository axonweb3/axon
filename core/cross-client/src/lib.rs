#![allow(dead_code, unused_variables, clippy::derive_partial_eq_without_eq)]

mod adapter;
mod error;
mod generated;
mod monitor;
mod task;
mod types;

pub use adapter::DefaultCrossAdapter;

use std::sync::Arc;

use arc_swap::ArcSwap;
use ckb_types::{core::TransactionView, prelude::*};

use common_crypto::{
    PrivateKey, PublicKey, Secp256k1RecoverablePrivateKey, Signature, ToPublicKey,
};
use protocol::tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use protocol::traits::{CkbClient, Context, CrossAdapter, CrossChain};
use protocol::types::{
    Block, BlockNumber, Hash, Hasher, Log, Proof, SignedTransaction, Transaction,
    TransactionAction, TypesError, UnverifiedTransaction, H160, H256, MAX_BLOCK_GAS_LIMIT, U256,
};
use protocol::{async_trait, lazy::CHAIN_ID, tokio, ProtocolResult};

use core_executor::CROSSCHAIN_CONTRACT_ADDRESS;

use crate::types::{Direction, Requests, Transfer};
use crate::{adapter::fixed_array, error::CrossChainError, monitor::CrossChainMonitor};

pub const CKB_BLOCK_INTERVAL: u64 = 8; // second
pub const NON_FORK_BLOCK_GAP: u64 = 24;

lazy_static::lazy_static! {
    pub static ref CKB_TIP: ArcSwap<u64> = ArcSwap::from_pointee(0);
}

pub struct CrossChainImpl<Adapter> {
    priv_key: Secp256k1RecoverablePrivateKey,
    address:  H160,
    log_rx:   UnboundedReceiver<Vec<Log>>,
    req_rx:   UnboundedReceiver<Vec<TransactionView>>,

    adapter: Arc<Adapter>,
}

impl<Adapter: CrossAdapter + 'static> CrossChainImpl<Adapter> {
    pub async fn new<C: CkbClient + 'static>(
        priv_key: Secp256k1RecoverablePrivateKey,
        acs_code_hash: H256,
        request_code_hash: H256,
        start_monitor_number: u64,
        ckb_client: Arc<C>,
        adapter: Arc<Adapter>,
    ) -> (Self, CrossChainHandler) {
        let address: H160 = Hasher::digest(priv_key.pub_key().to_bytes()).into();
        let (log_tx, log_rx) = unbounded_channel();
        let (req_tx, req_rx) = unbounded_channel();
        let init_monitor_number = adapter
            .get_monitor_ckb_number()
            .await
            .unwrap_or(start_monitor_number);

        tokio::spawn(async move {
            CrossChainMonitor::new(
                ckb_client,
                req_tx,
                init_monitor_number,
                acs_code_hash,
                request_code_hash,
            )
            .run()
            .await
        });

        let crosschain = CrossChainImpl {
            priv_key,
            address,
            log_rx,
            req_rx,
            adapter,
        };
        let handler = CrossChainHandler(log_tx);

        (crosschain, handler)
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(logs) = self.log_rx.recv() => {
                    let adapter_clone = Arc::clone(&self.adapter);

                    tokio::spawn(async move {
                        match build_ckb_txs(logs).await {
                            Ok((reqs, stx)) => {
                                let ctx = Context::new();
                                adapter_clone.insert_in_process(
                                    ctx.clone(),
                                    &rlp::encode(&reqs).freeze(),
                                    stx.pack().as_slice()
                                )
                                .await
                                .unwrap();
                                adapter_clone.send_ckb_tx(ctx, stx.into()).await;
                            },

                            Err(e) => log::error!("[crosschain]: crosschain error {:?}", e),
                        };

                    });
                }

                Some(reqs) = self.req_rx.recv() => {
                    let (reqs, stx) = match self.build_axon_txs(reqs).await {
                        Ok(res) => res,
                        Err(e) => {
                            log::error!("[crosschain]: crosschain error {:?}", e);
                            continue;
                        }
                    };

                    self.adapter.insert_in_process(
                        Context::new(),
                        &rlp::encode(&reqs).freeze(),
                        &rlp::encode(&stx).freeze()
                    )
                    .await
                    .unwrap();
                }
            }
        }
    }

    async fn build_axon_txs(
        &self,
        txs: Vec<TransactionView>,
    ) -> ProtocolResult<(Requests, SignedTransaction)> {
        let reqs = txs
            .iter()
            .map(|tx| {
                let type_script = tx.output(2).unwrap().type_().to_opt().unwrap();
                let request_args = generated::Transfer::new_unchecked(type_script.args().unpack());

                Transfer {
                    direction:      Direction::FromCkb,
                    tx_hash:        H256(tx.hash().unpack().0),
                    address:        H160::from_slice(&request_args.axon_address().raw_data()),
                    sudt_type_hash: H256(type_script.calc_script_hash().unpack().0),
                    erc20_address:  H160::from_slice(&request_args.e_r_c20_address().raw_data()),
                    ckb_amount:     u64::from_le_bytes(fixed_array(
                        &request_args.ckb_amount().raw_data(),
                    )),
                    sudt_amount:    u128::from_le_bytes(fixed_array(
                        &request_args.s_u_d_t_amount().raw_data(),
                    )),
                }
            })
            .collect::<Vec<_>>();

        let tx = Transaction {
            nonce:                    self.adapter.nonce(self.address).await?,
            max_priority_fee_per_gas: U256::one(),
            gas_price:                U256::one(),
            gas_limit:                MAX_BLOCK_GAS_LIMIT.into(),
            action:                   TransactionAction::Call(CROSSCHAIN_CONTRACT_ADDRESS),
            value:                    U256::zero(),
            data:                     Default::default(),
            access_list:              vec![],
        };

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

        let stx: SignedTransaction = utx
            .try_into()
            .map_err(|e: TypesError| CrossChainError::Adapter(e.to_string()))?;

        Ok((Requests(reqs), stx))
    }

    async fn complete_task(&self, logs: &[Log]) -> ProtocolResult<()> {
        for log in logs.iter() {
            // if
            self.adapter.remove_in_process(Context::new(), &[]).await?;
        }

        Ok(())
    }
}

pub struct CrossChainHandler(UnboundedSender<Vec<Log>>);

#[async_trait]
impl CrossChain for CrossChainHandler {
    async fn set_evm_log(
        &self,
        ctx: Context,
        block_number: BlockNumber,
        block_hash: Hash,
        logs: &[Vec<Log>],
    ) {
        for tx_logs in logs.iter() {
            if let Err(e) = self.0.send(tx_logs.clone()) {
                log::error!("[cross-chain]: send log to process error {:?}", e);
            }
        }
    }

    async fn set_checkpoint(&self, ctx: Context, block: Block, proof: Proof) {}
}

async fn build_ckb_txs(logs: Vec<Log>) -> ProtocolResult<(Requests, TransactionView)> {
    todo!()
}
