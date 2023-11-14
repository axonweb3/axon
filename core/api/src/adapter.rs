use std::sync::Arc;

use protocol::traits::{
    APIAdapter, Context, Executor, ExecutorReadOnlyAdapter, MemPool, Network, ReadOnlyStorage,
};
use protocol::trie::Trie as _;
use protocol::types::{
    Account, BigEndianHash, Block, BlockNumber, Bytes, CkbRelatedInfo, EthAccountProof,
    EthStorageProof, ExecutorContext, HardforkInfo, HardforkInfoInner, Hash, Hasher, Header, Hex,
    Metadata, Proposal, Receipt, SignedTransaction, TxResp, H160, H256, MAX_BLOCK_GAS_LIMIT,
    NIL_DATA, RLP_NULL, U256,
};
use protocol::{async_trait, codec::ProtocolCodec, trie, ProtocolError, ProtocolResult};

use core_executor::{
    system_contract::metadata::MetadataHandle, AxonExecutor, AxonExecutorReadOnlyAdapter, MPTTrie,
};

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
    S: ReadOnlyStorage + 'static,
    DB: trie::DB + Send + Sync + 'static,
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
    ) -> ProtocolResult<AxonExecutorReadOnlyAdapter<S, DB>> {
        let block = self
            .get_block_by_number(Context::new(), number)
            .await?
            .ok_or_else(|| APIError::Adapter(format!("Cannot get {:?} block", number)))?;

        AxonExecutorReadOnlyAdapter::from_root(
            block.header.state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            Proposal::new_without_state_root(&block.header).into(),
        )
    }
}

#[async_trait]
impl<M, S, DB, Net> APIAdapter for DefaultAPIAdapter<M, S, DB, Net>
where
    M: MemPool + 'static,
    S: ReadOnlyStorage + 'static,
    DB: trie::DB + Send + Sync + 'static,
    Net: Network + 'static,
{
    async fn insert_signed_txs(
        &self,
        ctx: Context,
        signed_tx: SignedTransaction,
    ) -> ProtocolResult<()> {
        self.mempool.insert(ctx, signed_tx).await
    }

    async fn mempool_contains_tx(&self, ctx: Context, tx_hash: &Hash) -> bool {
        self.mempool.contains(ctx, tx_hash).await
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

    async fn get_block_number_by_hash(
        &self,
        ctx: Context,
        hash: Hash,
    ) -> ProtocolResult<Option<BlockNumber>> {
        self.storage.get_block_number_by_hash(ctx, &hash).await
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
        if let Some(tx) = self.mempool.get_tx_from_mem(ctx.clone(), &tx_hash) {
            Ok(Some(tx))
        } else {
            self.storage.get_transaction_by_hash(ctx, &tx_hash).await
        }
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
        match self.evm_backend(number).await?.get(address.as_bytes()) {
            Some(bytes) => Account::decode(bytes),
            None => Ok(Account {
                nonce:        U256::zero(),
                balance:      U256::zero(),
                storage_root: RLP_NULL,
                code_hash:    NIL_DATA,
            }),
        }
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
        from: Option<H160>,
        to: Option<H160>,
        gas_price: Option<U256>,
        gas_limit: Option<U256>,
        value: U256,
        data: Vec<u8>,
        state_root: Hash,
        mock_header: Proposal,
    ) -> ProtocolResult<TxResp> {
        let mut exec_ctx = ExecutorContext::from(mock_header);
        exec_ctx.origin = from.unwrap_or_default();
        exec_ctx.gas_price = gas_price.unwrap_or_else(U256::one);

        let backend = AxonExecutorReadOnlyAdapter::from_root(
            state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            exec_ctx,
        )?;
        let gas_limit = gas_limit
            .map(|gas| gas.low_u64())
            .unwrap_or(MAX_BLOCK_GAS_LIMIT);

        Ok(AxonExecutor.call(&backend, gas_limit, from, to, value, data))
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
            .map(Into::into)
            .ok_or_else(|| APIError::Adapter("Can't find this position".to_string()).into())
    }

    async fn get_proof(
        &self,
        _ctx: Context,
        address: H160,
        storage_position: Vec<U256>,
        state_root: Hash,
    ) -> ProtocolResult<EthAccountProof> {
        let state_mpt_tree = MPTTrie::from_root(state_root, Arc::clone(&self.trie_db))?;
        let raw_account = state_mpt_tree
            .get(address.as_bytes())?
            .ok_or_else(|| APIError::Adapter("Can't find this address".to_string()))?;

        let account = Account::decode(raw_account).unwrap();

        let account_proof = state_mpt_tree
            .get_proof(address.as_bytes())?
            .into_iter()
            .map(Hex::encode)
            .collect();

        let storage_mpt_tree = MPTTrie::from_root(account.storage_root, Arc::clone(&self.trie_db))?;

        let mut storage_proofs = Vec::with_capacity(storage_position.len());

        for h in storage_position {
            let hash: Hash = BigEndianHash::from_uint(&h);
            let proof = EthStorageProof {
                key:   hash,
                value: storage_mpt_tree
                    .get(hash.as_bytes())?
                    .map(|v| H256::from_slice(&v))
                    .ok_or_else(|| {
                        Into::<ProtocolError>::into(APIError::Adapter(
                            "Can't find this position".to_string(),
                        ))
                    })?,
                proof: state_mpt_tree
                    .get_proof(hash.as_bytes())?
                    .into_iter()
                    .map(Hex::encode)
                    .collect(),
            };
            storage_proofs.push(proof);
        }

        Ok(EthAccountProof {
            balance: account.balance,
            code_hash: account.code_hash,
            nonce: account.nonce,
            storage_hash: Hasher::digest(account.storage_root),
            account_proof,
            storage_proof: storage_proofs,
        })
    }

    async fn get_metadata_by_number(
        &self,
        ctx: Context,
        block_number: Option<u64>,
    ) -> ProtocolResult<Metadata> {
        if let Some(num) = block_number {
            return MetadataHandle::new(self.get_metadata_root(ctx, Some(num)).await?)
                .get_metadata_by_block_number(num);
        }

        let num = self
            .storage
            .get_latest_block_header(ctx.clone())
            .await?
            .number;
        MetadataHandle::new(self.get_metadata_root(ctx, None).await?)
            .get_metadata_by_block_number(num)
    }

    async fn get_ckb_related_info(&self, ctx: Context) -> ProtocolResult<CkbRelatedInfo> {
        MetadataHandle::new(self.get_metadata_root(ctx, None).await?).get_ckb_related_info()
    }

    async fn get_image_cell_root(&self, ctx: Context) -> ProtocolResult<H256> {
        let state_root = self.storage.get_latest_block_header(ctx).await?.state_root;

        Ok(AxonExecutorReadOnlyAdapter::from_root(
            state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            Default::default(),
        )?
        .get_image_cell_root())
    }

    async fn get_metadata_root(&self, ctx: Context, number: Option<u64>) -> ProtocolResult<H256> {
        let state_root = match number {
            Some(n) => {
                self.storage
                    .get_block_header(ctx, n)
                    .await?
                    .ok_or_else(|| APIError::RequestPayload("Not found number".to_string()))?
                    .state_root
            }
            None => self.storage.get_latest_block_header(ctx).await?.state_root,
        };

        Ok(AxonExecutorReadOnlyAdapter::from_root(
            state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            Default::default(),
        )?
        .get_metadata_root())
    }

    async fn hardfork_info(&self, ctx: Context) -> ProtocolResult<HardforkInfo> {
        MetadataHandle::new(self.get_metadata_root(ctx, None).await?).hardfork_infos()
    }

    async fn hardfork_proposal(&self, ctx: Context) -> ProtocolResult<Option<HardforkInfoInner>> {
        self.storage.hardfork_proposal(ctx).await
    }
}
