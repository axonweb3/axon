use async_trait::async_trait;

use crate::traits::Context;
use crate::types::{Block, Hash, Header, Receipt, SignedTransaction};
use crate::ProtocolResult;

#[async_trait]
pub trait APIAdapter: Send + Sync {
    async fn insert_signed_txs(
        &self,
        ctx: Context,
        signed_tx: SignedTransaction,
    ) -> ProtocolResult<()>;

    async fn get_block_by_height(
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
}
