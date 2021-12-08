use async_trait::async_trait;
use creep::Context;

use crate::types::{Header, Transaction, Validator};
use crate::ProtocolResult;

#[async_trait]
pub trait CkbClient: Send + Sync {
    async fn get_validator_list(&self, ctx: Context) -> ProtocolResult<Vec<Validator>>;

    async fn watch_cross_tx(&self, ctx: Context) -> ProtocolResult<Transaction>;

    async fn verify_check_point(&self, ctx: Context, header: Header) -> ProtocolResult<()>;
}
