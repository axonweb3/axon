use crate::types::{
    Account, Block, BlockNumber, Bytes, Hash, Header, Proposal, Receipt, SignedTransaction, TxResp,
    H160, U256,
};
use crate::{async_trait, traits::Context, ProtocolResult};

#[async_trait]
pub trait APIAdapter: Send + Sync {
    async fn insert_signed_txs(
        &self,
        ctx: Context,
        signed_tx: SignedTransaction,
    ) -> ProtocolResult<()>;

    async fn get_block_by_number(
        &self,
        ctx: Context,
        height: Option<u64>,
    ) -> ProtocolResult<Option<Block>>;

    async fn get_block_by_hash(&self, ctx: Context, hash: Hash) -> ProtocolResult<Option<Block>>;

    async fn get_block_header_by_number(
        &self,
        ctx: Context,
        height: Option<u64>,
    ) -> ProtocolResult<Option<Header>>;

    async fn get_block_number_by_hash(
        &self,
        ctx: Context,
        hash: Hash,
    ) -> ProtocolResult<Option<BlockNumber>>;

    async fn get_receipt_by_tx_hash(
        &self,
        ctx: Context,
        tx_hash: Hash,
    ) -> ProtocolResult<Option<Receipt>>;

    async fn get_receipts_by_hashes(
        &self,
        ctx: Context,
        block_number: u64,
        tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<Receipt>>>;

    async fn get_transaction_by_hash(
        &self,
        ctx: Context,
        tx_hash: Hash,
    ) -> ProtocolResult<Option<SignedTransaction>>;

    async fn get_transactions_by_hashes(
        &self,
        ctx: Context,
        block_number: u64,
        tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<SignedTransaction>>>;

    async fn get_account(
        &self,
        ctx: Context,
        address: H160,
        number: Option<BlockNumber>,
    ) -> ProtocolResult<Account>;

    async fn get_pending_tx_count(&self, ctx: Context, address: H160) -> ProtocolResult<U256>;

    async fn evm_call(
        &self,
        ctx: Context,
        from: Option<H160>,
        to: Option<H160>,
        gas_price: Option<U256>,
        gas_limit: Option<U256>,
        value: U256,
        data: Vec<u8>,
        state_root: Hash,
        proposal: Proposal,
    ) -> ProtocolResult<TxResp>;

    async fn get_code_by_hash(&self, ctx: Context, hash: &Hash) -> ProtocolResult<Option<Bytes>>;

    async fn peer_count(&self, ctx: Context) -> ProtocolResult<U256>;

    async fn get_storage_at(
        &self,
        ctx: Context,
        address: H160,
        position: U256,
        state_root: Hash,
    ) -> ProtocolResult<Bytes>;
}
