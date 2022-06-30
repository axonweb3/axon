use ckb_jsonrpc_types::JsonBytes;
use ckb_sdk::rpc::ckb_indexer::{Cell, Pagination, SearchKey};
use ckb_types::core::TransactionView;

use common_crypto::{BlsPublicKey, BlsSignature};

use crate::async_trait;
use crate::traits::{Context, RPC};
use crate::types::{Transfer, H256};
use crate::ProtocolResult;

pub trait TxAssemblerAdapter: Send + Sync {
    fn fetch_live_cells(
        &self,
        ctx: Context,
        search_key: SearchKey,
        limit: u32,
        cursor: Option<JsonBytes>,
    ) -> RPC<Pagination<Cell>>;
}

#[async_trait]
pub trait TxAssembler: Sync + Send {
    async fn generate_crosschain_transaction_digest(
        &self,
        ctx: Context,
        transfers: &[Transfer],
    ) -> ProtocolResult<TransactionView>;

    fn complete_crosschain_transaction(
        &self,
        ctx: Context,
        digest: H256,
        bls_signature: &BlsSignature,
        bls_pubkey_list: &[BlsPublicKey],
    ) -> ProtocolResult<TransactionView>;
}
