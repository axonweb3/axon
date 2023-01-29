mod db;

pub use db::CrossChainDBImpl;

use std::sync::Arc;

use ckb_jsonrpc_types::OutputsValidator;
use ckb_types::core::TransactionView;

use common_crypto::{BlsPublicKey, BlsSignature};
use core_executor::{AxonExecutor, AxonExecutorAdapter};
use protocol::traits::{
    Backend, CkbClient, Context, CrossAdapter, Executor, MemPool, MessageTarget, MetadataControl,
    Storage, TxAssembler,
};
use protocol::types::{
    Metadata, RequestTxHashes, SignedTransaction, Transfer, TxResp, H160, H256, U256,
};
use protocol::{async_trait, lazy::CURRENT_STATE_ROOT, trie, ProtocolResult};

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
    TrieDB: trie::DB + 'static,
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

    async fn insert_in_process(&self, _ctx: Context, key: &[u8], val: &[u8]) -> ProtocolResult<()> {
        self.db.insert(key, val)
    }

    async fn get_in_process(&self, _ctx: Context, key: &[u8]) -> ProtocolResult<Option<Vec<u8>>> {
        self.db.get(key)
    }

    async fn get_all_in_process(&self, _ctx: Context) -> ProtocolResult<Vec<(Vec<u8>, Vec<u8>)>> {
        self.db.get_all()
    }

    async fn remove_in_process(&self, _ctx: Context, key: &[u8]) -> ProtocolResult<()> {
        self.db.remove(key)
    }

    async fn update_monitor_ckb_number(&self, ctx: Context, number: u64) -> ProtocolResult<()> {
        self.storage.update_monitor_ckb_number(ctx, number).await
    }

    async fn get_monitor_ckb_number(&self, ctx: Context) -> ProtocolResult<u64> {
        self.storage.get_monitor_ckb_number(ctx).await
    }

    async fn nonce(&self, _ctx: Context, address: H160) -> ProtocolResult<U256> {
        Ok(self.evm_backend()?.basic(address).nonce)
    }

    async fn call_evm(&self, ctx: Context, addr: H160, data: Vec<u8>) -> ProtocolResult<TxResp> {
        let header = self.storage.get_latest_block_header(ctx).await?;

        let backend = AxonExecutorAdapter::from_root(
            header.state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            (&header).into(),
        )?;

        Ok(AxonExecutor::default().call(
            &backend,
            u64::MAX,
            None,
            Some(addr),
            U256::default(),
            data,
        ))
    }

    async fn insert_record(
        &self,
        ctx: Context,
        reqs: RequestTxHashes,
        relay_tx_hash: H256,
    ) -> ProtocolResult<()> {
        let dir = reqs.direction;
        self.storage
            .insert_crosschain_records(ctx, reqs, relay_tx_hash, dir)
            .await
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
        _ctx: Context,
        _msg: Vec<u8>,
        _end: &str,
        _target: MessageTarget,
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
    TrieDB: trie::DB + 'static,
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
