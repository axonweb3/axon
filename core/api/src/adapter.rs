use protocol::{
    async_trait,
    traits::{APIAdapter, Context, MemPool, Storage},
    types::{Block, Hash, Header, Receipt, SignedTransaction},
    ProtocolResult,
};
use std::sync::Arc;

pub struct Adapter<M, S> {
    pool:  Arc<M>,
    store: Arc<S>,
}

impl<M, S> Adapter<M, S>
where
    M: MemPool + 'static,
    S: Storage + 'static,
{
    pub fn new(pool: Arc<M>, store: Arc<S>) -> Self {
        Self { pool, store }
    }
}

#[async_trait]
impl<M, S> APIAdapter for Adapter<M, S>
where
    M: MemPool + 'static,
    S: Storage + 'static,
{
    async fn insert_signed_txs(
        &self,
        ctx: Context,
        signed_tx: SignedTransaction,
    ) -> ProtocolResult<()> {
        self.pool.insert(ctx, signed_tx).await
    }

    async fn get_block_by_height(
        &self,
        ctx: Context,
        height: Option<u64>,
    ) -> ProtocolResult<Option<Block>> {
        match height {
            Some(number) => self.store.get_block(ctx, number).await,
            None => self.store.get_latest_block(ctx).await.map(Option::Some),
        }
    }

    async fn get_block_header_by_height(
        &self,
        ctx: Context,
        height: Option<u64>,
    ) -> ProtocolResult<Option<Header>> {
        match height {
            Some(number) => self.store.get_block_header(ctx, number).await,
            None => self
                .store
                .get_latest_block_header(ctx)
                .await
                .map(Option::Some),
        }
    }

    async fn get_receipt_by_tx_hash(
        &self,
        ctx: Context,
        tx_hash: Hash,
    ) -> ProtocolResult<Option<Receipt>> {
        self.store.get_receipt_by_hash(ctx, tx_hash).await
    }

    async fn get_transaction_by_hash(
        &self,
        ctx: Context,
        tx_hash: Hash,
    ) -> ProtocolResult<Option<SignedTransaction>> {
        self.store.get_transaction_by_hash(ctx, &tx_hash).await
    }
}
