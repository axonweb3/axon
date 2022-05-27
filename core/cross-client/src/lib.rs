#![allow(dead_code, unused_variables, clippy::derive_partial_eq_without_eq)]

mod adapter;
mod codec;
mod error;
mod generated;
mod monitor;
mod task;
mod types;

pub use adapter::DefaultCrossAdapter;

use std::sync::Arc;

use arc_swap::ArcSwap;
use ckb_types::{core::TransactionView, prelude::*};

use protocol::tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use protocol::traits::{Context, CrossAdapter, CrossChain};
use protocol::types::{Block, BlockNumber, Hash, Log, Proof, SignedTransaction};
use protocol::{async_trait, tokio};

use crate::types::Requests;

pub const CKB_BLOCK_INTERVAL: u64 = 8; // second
pub const NON_FORK_BLOCK_GAP: u64 = 24;

lazy_static::lazy_static! {
    pub static ref CKB_TIP: ArcSwap<u64> = ArcSwap::from_pointee(0);
}

pub struct CrossChainImpl<Adapter> {
    adapter: Arc<Adapter>,
    log_rx:  UnboundedReceiver<Vec<Log>>,
    req_rx:  UnboundedReceiver<Vec<TransactionView>>,
}

#[async_trait]
impl<Adapter: CrossAdapter + 'static> CrossChain for CrossChainImpl<Adapter> {
    async fn set_evm_log(
        &self,
        ctx: Context,
        block_number: BlockNumber,
        block_hash: Hash,
        logs: &[Vec<Log>],
    ) {
        for tx_logs in logs.iter() {
            if let Err(e) = self.log_tx.send(tx_logs.clone()) {
                log::error!("[cross-chain]: send log to process error {:?}", e);
            }
        }
    }

    async fn set_checkpoint(&self, ctx: Context, block: Block, proof: Proof) {}
}

impl<Adapter: CrossAdapter + 'static> CrossChainImpl<Adapter> {
    pub fn new(adapter: Arc<Adapter>) -> Self {
        let (log_tx, log_rx) = unbounded_channel();
        let (req_tx, req_rx) = unbounded_channel();
        CrossChainImpl {
            adapter,
            log_rx,
            req_rx,
        }
    }

    pub async fn run(mut self) {
        tokio::select! {
            Some(logs) = self.log_rx.recv() => {
                let adapter_clone = Arc::clone(&self.adapter);

                tokio::spawn(async move {
                    let ctx = Context::new();
                    let (reqs, stx) = build_ckb_txs(logs);
                    adapter_clone.insert_in_process(
                        ctx.clone(),
                        &rlp::encode(&reqs).freeze(),
                        stx.pack().as_slice()
                    );
                    adapter_clone.send_ckb_tx(ctx, stx.into()).await;
                });
            }

            Some(reqs) = self.req_rx.recv() => {
                let (reqs, stx) = self.build_axon_txs(reqs);
                self.adapter.insert_in_process(
                    Context::new(),
                    &rlp::encode(&reqs).freeze(),
                        &rlp::encode(&stx).freeze()
                );
            }
        }
    }

    fn build_axon_txs(&self, txs: Vec<TransactionView>) -> (Requests, SignedTransaction) {
        todo!()
    }
}

fn build_ckb_txs(logs: Vec<Log>) -> (Requests, TransactionView) {
    todo!()
}
