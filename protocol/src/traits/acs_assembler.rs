use ckb_types::core::TransactionView;
use common_crypto::{BlsPublicKey, BlsSignature};

use crate::async_trait;
use crate::types::{Transfer, H256};
use crate::{traits::Context, ProtocolResult};

#[async_trait]
pub trait AcsAssembler: Sync + Send {
    async fn generate_crosschain_transaction_digest(
        &self,
        ctx: Context,
        transfers: &[Transfer],
    ) -> ProtocolResult<H256>;

    fn complete_crosschain_transaction(
        &self,
        ctx: Context,
        digest: H256,
        bls_signature: &BlsSignature,
        bls_pubkey_list: &[BlsPublicKey],
    ) -> ProtocolResult<TransactionView>;
}
