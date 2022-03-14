use std::sync::Arc;

use core_executor::{is_call_system_script, AxonExecutor, AxonExecutorAdapter, MPTTrie};
use protocol::traits::{APIAdapter, Context, Executor, ExecutorAdapter, MemPool, Network, Storage};
use protocol::types::{
    Account, BigEndianHash, Block, BlockNumber, Bytes, ExecutorContext, Hash, Header, Proposal,
    Receipt, SignedTransaction, TxResp, H160, U256,
};
use protocol::{async_trait, codec::ProtocolCodec, ProtocolResult};

use crate::APIError;

#[derive(Clone)]
pub struct DefaultAPIAdapter<M, S, DB, Net> {
    mempool: Arc<M>,
    storage: Arc<S>,
    trie_db: Arc<DB>,
    net:     Arc<Net>,
}

impl<M, S, DB, Net> DefaultAPIAdapter<M, S, DB, Net>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
    Net: Network + 'static,
{
    pub fn new(mempool: Arc<M>, storage: Arc<S>, trie_db: Arc<DB>, net: Arc<Net>) -> Self {
        Self {
            mempool,
            storage,
            trie_db,
            net,
        }
    }

    pub async fn evm_backend(
        &self,
        number: Option<BlockNumber>,
    ) -> ProtocolResult<AxonExecutorAdapter<S, DB>> {
        let block = self
            .get_block_by_number(Context::new(), number)
            .await?
            .ok_or_else(|| APIError::Adapter(format!("Cannot get {:?} block", number)))?;
        let state_root = block.header.state_root;
        let proposal: Proposal = block.into();

        AxonExecutorAdapter::from_root(
            state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            ExecutorContext::from(proposal),
        )
    }
}

#[async_trait]
impl<M, S, DB, Net> APIAdapter for DefaultAPIAdapter<M, S, DB, Net>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
    Net: Network + 'static,
{
    async fn insert_signed_txs(
        &self,
        ctx: Context,
        signed_tx: SignedTransaction,
    ) -> ProtocolResult<()> {
        if is_call_system_script(&signed_tx.transaction.unsigned.action) {
            return self.mempool.insert_system_script_tx(ctx, signed_tx).await;
        }

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

    async fn get_block_by_hash(&self, ctx: Context, hash: Hash) -> ProtocolResult<Option<Block>> {
        self.storage.get_block_by_hash(ctx, &hash).await
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
        self.storage.get_receipt_by_hash(ctx, &tx_hash).await
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

    async fn get_pending_tx_count(&self, ctx: Context, address: H160) -> ProtocolResult<U256> {
        self.mempool
            .get_tx_count_by_address(ctx, address)
            .await
            .map(U256::from)
    }

    async fn evm_call(
        &self,
        _ctx: Context,
        address: H160,
        data: Vec<u8>,
        state_root: Hash,
        mock_header: Proposal,
    ) -> ProtocolResult<TxResp> {
        let mut backend = AxonExecutorAdapter::from_root(
            state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            ExecutorContext::from(mock_header),
        )?;

        Ok(AxonExecutor::default().call(&mut backend, address, data))
    }

    async fn get_code_by_hash(&self, ctx: Context, hash: &Hash) -> ProtocolResult<Option<Bytes>> {
        self.storage.get_code_by_hash(ctx, hash).await
    }

    async fn peer_count(&self, ctx: Context) -> ProtocolResult<U256> {
        self.net.peer_count(ctx).map(Into::into)
    }

    async fn get_storage_at(
        &self,
        _ctx: Context,
        address: H160,
        position: U256,
        state_root: Hash,
    ) -> ProtocolResult<Bytes> {
        let state_mpt_tree = MPTTrie::from_root(state_root, Arc::clone(&self.trie_db))?;

        let raw_account = state_mpt_tree
            .get(address.as_bytes())?
            .ok_or_else(|| APIError::Adapter("Can't find this address".to_string()))?;

        let account = Account::decode(raw_account).unwrap();

        let storage_mpt_tree = MPTTrie::from_root(account.storage_root, Arc::clone(&self.trie_db))?;

        let hash: Hash = BigEndianHash::from_uint(&position);
        storage_mpt_tree
            .get(hash.as_bytes())?
            .ok_or_else(|| APIError::Adapter("Can't find this position".to_string()).into())
    }
}
