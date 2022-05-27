use ckb_jsonrpc_types::TransactionView;

use crate::types::{Block, BlockNumber, Hash, Log, Proof, SignedTransaction};
use crate::{async_trait, traits::Context, ProtocolResult};

#[async_trait]
pub trait CrossAdapter: Send + Sync {
    async fn send_axon_tx(&self, ctx: Context, stx: SignedTransaction) -> ProtocolResult<()>;

    async fn send_ckb_tx(&self, ctx: Context, tx: TransactionView) -> ProtocolResult<()>;

    fn insert_in_process(&self, ctx: Context, key: &[u8], val: &[u8]) -> ProtocolResult<()>;

    fn get_all_in_process(&self, ctx: Context) -> ProtocolResult<Vec<(Vec<u8>, Vec<u8>)>>;

    fn remove_in_process(&self, ctx: Context, key: &[u8]) -> ProtocolResult<()>;
}

#[async_trait]
pub trait CrossChain: Send + Sync {
    async fn set_evm_log(
        &self,
        ctx: Context,
        block_number: BlockNumber,
        block_hash: Hash,
        logs: &[Vec<Log>],
    );

    async fn set_checkpoint(&self, ctx: Context, block: Block, proof: Proof);
}
