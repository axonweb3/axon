use crate::types::{
    BlockNumber, Hash, MerkleRoot, PackedTxHashes, SignedTransaction, H160, U256, U64,
};
use crate::{async_trait, traits::Context, ProtocolResult};

#[async_trait]
pub trait MemPool: Send + Sync {
    async fn insert(&self, ctx: Context, tx: SignedTransaction) -> ProtocolResult<()>;

    async fn contains(&self, ctx: Context, tx_hash: &Hash) -> bool;

    async fn package(
        &self,
        ctx: Context,
        cycles_limit: U256,
        tx_num_limit: u64,
    ) -> ProtocolResult<PackedTxHashes>;

    async fn flush(
        &self,
        ctx: Context,
        tx_hashes: &[Hash],
        current_number: BlockNumber,
    ) -> ProtocolResult<()>;

    async fn get_full_txs(
        &self,
        ctx: Context,
        height: Option<u64>,
        tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<SignedTransaction>>;

    async fn ensure_order_txs(
        &self,
        ctx: Context,
        height: Option<u64>,
        order_tx_hashes: &[Hash],
    ) -> ProtocolResult<()>;

    async fn get_tx_count_by_address(
        &self,
        ctx: Context,
        address: H160,
    ) -> ProtocolResult<(usize, Option<BlockNumber>)>;

    fn get_tx_from_mem(&self, ctx: Context, tx_hash: &Hash) -> Option<SignedTransaction>;
    fn set_args(&self, context: Context, state_root: MerkleRoot, gas_limit: u64, max_tx_size: u64);
}

#[async_trait]
pub trait MemPoolAdapter: Send + Sync {
    async fn pull_txs(
        &self,
        ctx: Context,
        height: Option<u64>,
        tx_hashes: Vec<Hash>,
    ) -> ProtocolResult<Vec<SignedTransaction>>;

    async fn broadcast_tx(
        &self,
        ctx: Context,
        origin: Option<usize>,
        tx: SignedTransaction,
    ) -> ProtocolResult<()>;

    async fn check_authorization(
        &self,
        ctx: Context,
        tx: &SignedTransaction,
    ) -> ProtocolResult<U64>;

    async fn check_transaction(&self, ctx: Context, tx: &SignedTransaction) -> ProtocolResult<()>;

    async fn check_storage_exist(&self, ctx: Context, tx_hash: &Hash) -> ProtocolResult<()>;

    async fn get_latest_height(&self, ctx: Context) -> ProtocolResult<u64>;

    async fn get_transactions_from_storage(
        &self,
        ctx: Context,
        block_height: Option<u64>,
        tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<SignedTransaction>>>;

    fn set_args(&self, context: Context, state_root: MerkleRoot, gas_limit: u64, max_tx_size: u64);

    fn clear_nonce_cache(&self);

    fn report_good(&self, ctx: Context);
}
