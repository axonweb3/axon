use std::sync::Arc;

use core_executor::{EVMExecutorAdapter, EvmExecutor};
use protocol::traits::{APIAdapter, Context, Executor, ExecutorAdapter, MemPool, Storage};
use protocol::types::{
    Account, Block, BlockNumber, Bytes, ExecutorContext, Hash, Header, Receipt, SignedTransaction,
    TxResp, H160,
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
    ) -> ProtocolResult<EVMExecutorAdapter<S, DB>> {
        let block = self
            .get_block_by_number(Context::new(), number)
            .await?
            .ok_or_else(|| APIError::Adapter(format!("Cannot get {:?} block", number)))?;

        EVMExecutorAdapter::from_root(
            block.header.state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            ExecutorContext::from(block.header),
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

    async fn get_block_header_by_number(
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

    async fn get_receipts_by_hashes(
        &self,
        ctx: Context,
        block_number: u64,
        tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<Receipt>>> {
        self.storage
            .get_receipts(ctx, block_number, tx_hashes)
            .await
    }

    async fn get_transaction_by_hash(
        &self,
        ctx: Context,
        tx_hash: Hash,
    ) -> ProtocolResult<Option<SignedTransaction>> {
        self.storage.get_transaction_by_hash(ctx, &tx_hash).await
    }

    async fn get_transactions_by_hashes(
        &self,
        ctx: Context,
        block_number: u64,
        tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<SignedTransaction>>> {
        self.storage
            .get_transactions(ctx, block_number, tx_hashes)
            .await
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
            .ok_or_else(|| APIError::Adapter(format!("Cannot get {:?} account", address)))?;
        Account::decode(bytes)
    }

    async fn evm_call(
        &self,
        _ctx: Context,
        address: H160,
        data: Vec<u8>,
        mock_header: Header,
    ) -> ProtocolResult<TxResp> {
        let mut backend = EVMExecutorAdapter::from_root(
            mock_header.state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            ExecutorContext::from(mock_header),
        )?;

        Ok(EvmExecutor::default().call(&mut backend, address, data))
    }

    async fn get_code_by_hash(&self, ctx: Context, hash: &Hash) -> ProtocolResult<Option<Bytes>> {
        self.storage.get_code_by_hash(ctx, hash).await
    }
}
