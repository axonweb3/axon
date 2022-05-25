#![allow(
    dead_code,
    unused_variables,
    clippy::needless_return,
    clippy::derive_partial_eq_without_eq
)]

mod adapter;
mod codec;
mod error;
mod generated;
mod monitor;
mod types;

pub use adapter::DefaultCrossAdapter;

use std::sync::Arc;

use ckb_types::core::BlockView;

use protocol::async_trait;
use protocol::tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use protocol::traits::{Context, CrossAdapter, CrossChain};
use protocol::types::{Block, BlockNumber, Hash, Log, Proof};

pub struct CrossChainImpl<Adapter> {
    adapter: Arc<Adapter>,
    log_tx:  UnboundedSender<Vec<Log>>,
    req_tx:  UnboundedSender<Vec<BlockView>>,
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
            log_tx,
            req_tx,
        }
    }

    pub fn run(&self) {}
}
