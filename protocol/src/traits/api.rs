use crate::traits::Context;
use crate::types::{
    Account, Block, BlockNumber, Bytes, Hash, Header, Receipt, SignedTransaction, TxResp, H160,
};
use crate::ProtocolResult;
use async_trait::async_trait;

#[async_trait]
pub trait APIAdapter: Send + Sync {
    async fn insert_signed_txs(
        &self,
        ctx: Context,
        signed_tx: SignedTransaction,
    ) -> ProtocolResult<()>;

    async fn get_block_by_number(
        &self,
        ctx: Context,
        height: Option<u64>,
    ) -> ProtocolResult<Option<Block>>;

    async fn get_block_header_by_height(
        &self,
        ctx: Context,
        height: Option<u64>,
    ) -> ProtocolResult<Option<Header>>;

    async fn get_receipt_by_tx_hash(
        &self,
        ctx: Context,
        tx_hash: Hash,
    ) -> ProtocolResult<Option<Receipt>>;

    async fn get_transaction_by_hash(
        &self,
        ctx: Context,
        tx_hash: Hash,
    ) -> ProtocolResult<Option<SignedTransaction>>;

    async fn get_latest_block(&self, ctx: Context) -> ProtocolResult<Block>;

    async fn get_account(
        &self,
        ctx: Context,
        address: H160,
        number: Option<BlockNumber>,
    ) -> ProtocolResult<Account>;

    async fn evm_call(&self, ts: SignedTransaction) -> TxResp;

    async fn get_code_by_hash(&self, ctx: Context, hash: &Hash) -> ProtocolResult<Option<Bytes>>;
}
