use async_trait::async_trait;
use common_apm_derive::trace_span;

use protocol::{traits::Context, types::SignedTransaction, ProtocolResult};

#[async_trait]
pub trait ToTrace {
    async fn store(&self, ctx: Context, txs: Vec<SignedTransaction>) -> ProtocolResult<()>;

    fn version(&self, ctx: Context) -> String;
}

pub struct TraceExample;

#[async_trait]
impl ToTrace for TraceExample {
    #[trace_span(kind = "trace", logs = "{tx_len: txs.len()}")]
    async fn store(&self, ctx: Context, txs: Vec<SignedTransaction>) -> ProtocolResult<()> {
        debug_assert!(txs.len() == 1);
        Ok(())
    }

    #[trace_span(kind = "trace")]
    fn version(&self, ctx: Context) -> String {
        "0.1.0".to_string()
    }
}

fn main() {}
