mod db;

pub use db::CrossChainDBImpl;

use std::sync::Arc;

use core_executor::{AxonExecutor, AxonExecutorAdapter};
use protocol::traits::{Backend, Context, CrossAdapter, Executor, MemPool, Storage};
use protocol::types::{ExecutorContext, Proposal, SignedTransaction, TxResp, H160, U256};
use protocol::{async_trait, ProtocolResult};

use crate::error::CrossChainError;

const MONITOR_CKB_NUMBER_KEY: &str = "MonitorCkbNumberKey";

pub trait CrossChainDB: Sync + Send {
    fn get(&self, key: &[u8]) -> ProtocolResult<Option<Vec<u8>>>;

    fn get_all(&self) -> ProtocolResult<Vec<(Vec<u8>, Vec<u8>)>>;

    fn insert(&self, key: &[u8], val: &[u8]) -> ProtocolResult<()>;

    fn remove(&self, key: &[u8]) -> ProtocolResult<()>;
}

pub struct DefaultCrossChainAdapter<M, S, TrieDB, DB> {
    mempool: Arc<M>,
    storage: Arc<S>,
    trie_db: Arc<TrieDB>,
    db:      Arc<DB>,
}

#[async_trait]
impl<M, S, TrieDB, DB> CrossAdapter for DefaultCrossChainAdapter<M, S, TrieDB, DB>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    TrieDB: cita_trie::DB + 'static,
    DB: CrossChainDB + 'static,
{
    async fn send_axon_tx(&self, ctx: Context, stx: SignedTransaction) -> ProtocolResult<()> {
        self.mempool.insert(ctx, stx).await
    }

    async fn send_ckb_tx(
        &self,
        ctx: Context,
        tx: ckb_jsonrpc_types::TransactionView,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    async fn insert_in_process(&self, ctx: Context, key: &[u8], val: &[u8]) -> ProtocolResult<()> {
        self.db.insert(key, val)
    }

    async fn get_all_in_process(&self, ctx: Context) -> ProtocolResult<Vec<(Vec<u8>, Vec<u8>)>> {
        self.db.get_all()
    }

    async fn remove_in_process(&self, ctx: Context, key: &[u8]) -> ProtocolResult<()> {
        self.db.remove(key)
    }

    async fn update_monitor_ckb_number(&self, ctx: Context, number: u64) -> ProtocolResult<()> {
        self.db
            .insert(MONITOR_CKB_NUMBER_KEY.as_bytes(), &number.to_le_bytes())
    }

    async fn get_monitor_ckb_number(&self, _ctx: Context) -> ProtocolResult<u64> {
        match self.db.get(MONITOR_CKB_NUMBER_KEY.as_bytes()) {
            Ok(Some(bytes)) => Ok(u64::from_le_bytes(fixed_array(&bytes))),
            _ => Err(CrossChainError::Adapter("Cannot get monitor CKB number".to_string()).into()),
        }
    }

    async fn nonce(&self, _ctx: Context, address: H160) -> ProtocolResult<U256> {
        Ok(self.evm_backend().await?.basic(address).nonce)
    }

    async fn call_evm(&self, ctx: Context, addr: H160, data: Vec<u8>) -> ProtocolResult<TxResp> {
        let header = self.storage.get_latest_block_header(ctx).await?;

        let mut backend = AxonExecutorAdapter::from_root(
            header.state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            (&header).into(),
        )?;

        Ok(AxonExecutor::default().call(&mut backend, None, Some(addr), data))
    }
}

impl<M, S, TrieDB, DB> DefaultCrossChainAdapter<M, S, TrieDB, DB>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    TrieDB: cita_trie::DB + 'static,
    DB: CrossChainDB + 'static,
{
    pub async fn new(mempool: Arc<M>, storage: Arc<S>, trie_db: Arc<TrieDB>, db: Arc<DB>) -> Self {
        DefaultCrossChainAdapter {
            mempool,
            storage,
            trie_db,
            db,
        }
    }

    async fn evm_backend(&self) -> ProtocolResult<AxonExecutorAdapter<S, TrieDB>> {
        let block = self.storage.get_latest_block(Context::new()).await?;
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

pub fn fixed_array<const LEN: usize>(bytes: &[u8]) -> [u8; LEN] {
    assert_eq!(bytes.len(), LEN);
    let mut list = [0; LEN];
    list.copy_from_slice(bytes);
    list
}
