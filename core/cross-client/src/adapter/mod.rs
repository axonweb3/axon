mod db;

pub use db::CrossChainDBImpl;
use protocol::lazy::CURRENT_STATE_ROOT;

use std::sync::Arc;

use ckb_jsonrpc_types::OutputsValidator;
use ckb_types::core::TransactionView;

use common_crypto::{BlsPublicKey, BlsSignature};
use protocol::traits::{
    Backend, CkbClient, Context, CrossAdapter, Executor, MemPool, MessageTarget, MetadataControl,
    Storage, TxAssembler,
};
use protocol::types::{
    Metadata, RequestTxHashes, SignedTransaction, Transfer, TxResp, H160, H256, U256,
};
use protocol::{async_trait, ProtocolResult};

use core_executor::{AxonExecutor, AxonExecutorAdapter};

use crate::error::CrossChainError;

const MONITOR_CKB_NUMBER_KEY: &str = "MonitorCkbNumberKey";

pub trait CrossChainDB: Sync + Send {
    fn get(&self, key: &[u8]) -> ProtocolResult<Option<Vec<u8>>>;

    fn get_all(&self) -> ProtocolResult<Vec<(Vec<u8>, Vec<u8>)>>;

    fn insert(&self, key: &[u8], val: &[u8]) -> ProtocolResult<()>;

    fn remove(&self, key: &[u8]) -> ProtocolResult<()>;
}

pub struct DefaultCrossChainAdapter<M, D, S, A, TrieDB, DB, Rpc> {
    mempool:      Arc<M>,
    metadata:     Arc<D>,
    storage:      Arc<S>,
    tx_assembler: Arc<A>,
    trie_db:      Arc<TrieDB>,
    db:           Arc<DB>,
    ckb_rpc:      Arc<Rpc>,
}

#[async_trait]
impl<M, D, S, A, TrieDB, DB, Rpc> CrossAdapter
    for DefaultCrossChainAdapter<M, D, S, A, TrieDB, DB, Rpc>
where
    M: MemPool + 'static,
    D: MetadataControl + 'static,
    S: Storage + 'static,
    A: TxAssembler + 'static,
    TrieDB: cita_trie::DB + 'static,
    DB: CrossChainDB + 'static,
    Rpc: CkbClient + 'static,
{
    async fn send_axon_tx(&self, ctx: Context, stx: SignedTransaction) -> ProtocolResult<()> {
        self.mempool.insert(ctx, stx).await
    }

    async fn send_ckb_tx(
        &self,
        ctx: Context,
        tx: ckb_jsonrpc_types::TransactionView,
    ) -> ProtocolResult<()> {
        log::info!("[cross-chain]: send transaction to ckb {:?}", tx);

        let _hash = self
            .ckb_rpc
            .send_transaction(ctx, &tx.inner, Some(OutputsValidator::Passthrough))
            .await?;
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
        Ok(self.evm_backend()?.basic(address).nonce)
    }

    async fn call_evm(&self, ctx: Context, addr: H160, data: Vec<u8>) -> ProtocolResult<TxResp> {
        let header = self.storage.get_latest_block_header(ctx).await?;

        let mut backend = AxonExecutorAdapter::from_root(
            header.state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            (&header).into(),
        )?;

        Ok(AxonExecutor::default().call(&mut backend, u64::MAX, None, Some(addr), data))
    }

    async fn insert_record(
        &self,
        ctx: Context,
        reqs: RequestTxHashes,
        block_hash: H256,
    ) -> ProtocolResult<()> {
        self.storage
            .insert_crosschain_record(ctx, reqs, block_hash)
            .await
    }

    async fn get_record(
        &self,
        ctx: Context,
        reqs: RequestTxHashes,
    ) -> ProtocolResult<Option<H256>> {
        self.storage.get_crosschain_record(ctx, reqs).await
    }

    async fn current_metadata(&self, ctx: Context) -> Metadata {
        let number = self
            .storage
            .get_latest_block_header(ctx.clone())
            .await
            .unwrap()
            .number;
        self.metadata.get_metadata_unchecked(ctx, number)
    }

    async fn calc_to_ckb_tx(
        &self,
        ctx: Context,
        transfers: &[Transfer],
    ) -> ProtocolResult<TransactionView> {
        self.tx_assembler
            .generate_crosschain_transaction_digest(ctx, transfers)
            .await
    }

    fn build_to_ckb_tx(
        &self,
        ctx: Context,
        digest: H256,
        bls_signature: &BlsSignature,
        bls_pubkey_list: &[BlsPublicKey],
    ) -> ProtocolResult<TransactionView> {
        self.tx_assembler.complete_crosschain_transaction(
            ctx,
            digest,
            bls_signature,
            bls_pubkey_list,
        )
    }

    async fn transmit(
        &self,
        ctx: Context,
        msg: Vec<u8>,
        end: &str,
        target: MessageTarget,
    ) -> ProtocolResult<()> {
        Ok(())
    }
}

impl<M, D, S, A, TrieDB, DB, Rpc> DefaultCrossChainAdapter<M, D, S, A, TrieDB, DB, Rpc>
where
    M: MemPool + 'static,
    D: MetadataControl + 'static,
    S: Storage + 'static,
    A: TxAssembler + 'static,
    TrieDB: cita_trie::DB + 'static,
    DB: CrossChainDB + 'static,
    Rpc: CkbClient + 'static,
{
    pub async fn new(
        mempool: Arc<M>,
        metadata: Arc<D>,
        storage: Arc<S>,
        tx_assembler: Arc<A>,
        trie_db: Arc<TrieDB>,
        db: Arc<DB>,
        ckb_rpc: Arc<Rpc>,
    ) -> Self {
        DefaultCrossChainAdapter {
            mempool,
            metadata,
            storage,
            tx_assembler,
            trie_db,
            db,
            ckb_rpc,
        }
    }

    fn evm_backend(&self) -> ProtocolResult<AxonExecutorAdapter<S, TrieDB>> {
        AxonExecutorAdapter::from_root(
            **CURRENT_STATE_ROOT.load(),
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            Default::default(),
        )
    }
}

pub fn fixed_array<const LEN: usize>(bytes: &[u8]) -> [u8; LEN] {
    assert_eq!(bytes.len(), LEN);
    let mut list = [0; LEN];
    list.copy_from_slice(bytes);
    list
}
