use std::sync::Arc;

use core_executor::{EVMExecutorAdapter, EvmExecutor};
use protocol::traits::{Context, Executor, MetadataControlAdapter, Storage};
use protocol::types::{Header, TxResp, H160};
use protocol::ProtocolResult;

pub struct MetadataAdapterImpl<S, DB> {
    storage: Arc<S>,
    trie_db: Arc<DB>,
}

impl<S, DB> MetadataControlAdapter for MetadataAdapterImpl<S, DB>
where
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    fn call_evm(
        &self,
        _ctx: Context,
        header: &Header,
        addr: H160,
        data: Vec<u8>,
    ) -> ProtocolResult<TxResp> {
        let mut backend = EVMExecutorAdapter::from_root(
            header.state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            header.into(),
        )?;

        Ok(EvmExecutor::default().call(&mut backend, addr, data))
    }
}

impl<S, DB> MetadataAdapterImpl<S, DB>
where
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    pub fn new(storage: Arc<S>, trie_db: Arc<DB>) -> Self {
        MetadataAdapterImpl { storage, trie_db }
    }
}
