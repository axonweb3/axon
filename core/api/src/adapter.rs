use std::collections::BTreeMap;
use std::sync::Arc;

use core_executor::{EVMExecutorAdapter, EvmExecutor};
use protocol::traits::{APIAdapter, Context, ExecutorAdapter, MemPool, Storage};
use protocol::types::{
    Account, Block, BlockNumber, Bytes, ExecutorContext, Hash, Header, MemoryAccount,
    MemoryBackend, MemoryVicinity, Receipt, SignedTransaction, TxResp, H160, U256,
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
            .ok_or_else(|| APIError::AdapterError(format!("Cannot get {:?} block", number)))?;

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

    async fn evm_call(&self, ts: SignedTransaction) -> TxResp {
        let mut state = BTreeMap::new();

        state.insert(ts.sender, MemoryAccount {
            nonce:   U256::one(),
            balance: U256::max_value(),
            storage: BTreeMap::new(),
            code:    Vec::new(),
        });
        let mv = MemoryVicinity {
            gas_price:              U256::zero(),
            origin:                 H160::default(),
            block_hashes:           Vec::new(),
            block_number:           Default::default(),
            block_coinbase:         Default::default(),
            block_timestamp:        Default::default(),
            block_difficulty:       Default::default(),
            block_gas_limit:        Default::default(),
            chain_id:               U256::one(),
            block_base_fee_per_gas: U256::zero(),
        };

        let mut backend = MemoryBackend::new(&mv, state);
        let executor = EvmExecutor::new();
        let r = executor.inner_exec(&mut backend, ts);
        r
    }

    async fn get_code_by_hash(&self, ctx: Context, hash: &Hash) -> ProtocolResult<Option<Bytes>> {
        self.storage.get_code_by_hash(ctx, hash).await
    }
}
