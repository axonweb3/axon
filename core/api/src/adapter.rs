use std::sync::Arc;

use parking_lot::Mutex;

use core_executor::EVMExecutorAdapter;
use protocol::traits::{APIAdapter, Context, ExecutorAdapter, MemPool, Storage};
use protocol::types::{
    Account, Block, BlockNumber, ExecutorContext, Hash, Header, Receipt, SignedTransaction, H160,
};
use protocol::{async_trait, codec::ProtocolCodec, ProtocolResult};

use crate::APIError;

#[derive(Clone)]
pub struct DefaultAPIAdapter<M, S, DB> {
    mempool: Arc<M>,
    storage: Arc<S>,
    trie_db: Arc<DB>,
}

impl<M, S, DB> DefaultAPIAdapter<M, S, DB>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    pub fn new(mempool: Arc<M>, storage: Arc<S>, trie_db: Arc<DB>) -> Self {
        Self {
            mempool,
            storage,
            trie_db,
        }
    }

    pub async fn evm_backend(
        &self,
        number: Option<BlockNumber>,
    ) -> ProtocolResult<EVMExecutorAdapter<DB>> {
        let block = self
            .get_block_by_number(Context::new(), number)
            .await?
            .ok_or_else(|| APIError::AdapterError(format!("Cannot get {:?} block", number)))?;
        let state_root = block.header.state_root;
        let exec_ctx = ExecutorContext::from(block.header);
        EVMExecutorAdapter::from_root(
            state_root,
            Arc::clone(&self.trie_db),
            Arc::new(Mutex::new(exec_ctx)),
        )
    }
}

#[async_trait]
impl<M, S, DB> APIAdapter for DefaultAPIAdapter<M, S, DB>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    async fn insert_signed_txs(
        &self,
        ctx: Context,
        signed_tx: SignedTransaction,
    ) -> ProtocolResult<()> {
        self.mempool.insert(ctx, signed_tx).await
    }

    async fn get_block_by_number(
        &self,
        ctx: Context,
        height: Option<u64>,
    ) -> ProtocolResult<Option<Block>> {
        match height {
            Some(number) => self.storage.get_block(ctx, number).await,
            None => self.storage.get_latest_block(ctx).await.map(Option::Some),
        }
    }

    async fn get_block_header_by_height(
        &self,
        ctx: Context,
        number: Option<u64>,
    ) -> ProtocolResult<Option<Header>> {
        match number {
            Some(num) => self.storage.get_block_header(ctx, num).await,
            None => self
                .storage
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
        self.storage.get_receipt_by_hash(ctx, tx_hash).await
    }

    async fn get_transaction_by_hash(
        &self,
        ctx: Context,
        tx_hash: Hash,
    ) -> ProtocolResult<Option<SignedTransaction>> {
        self.storage.get_transaction_by_hash(ctx, &tx_hash).await
    }

    async fn get_latest_block(&self, ctx: Context) -> ProtocolResult<Block> {
        self.storage.get_latest_block(ctx).await
    }

    async fn get_account(
        &self,
        _ctx: Context,
        address: H160,
        number: Option<BlockNumber>,
    ) -> ProtocolResult<Account> {
        let bytes = self
            .evm_backend(number)
            .await?
            .get(address.as_bytes())
            .ok_or_else(|| APIError::AdapterError(format!("Cannot get {:?} account", address)))?;
        Account::decode(bytes)
    }
}
