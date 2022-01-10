mod pubsub;
mod stream_codec;

use std::sync::Arc;

use protocol::traits::{Context, CrossAdapter, MemPool};
use protocol::types::SignedTransaction;
use protocol::{async_trait, ProtocolResult};

pub struct DefaultCrossAdapter<M> {
    mempool: Arc<M>,
}

#[async_trait]
impl<M> CrossAdapter for DefaultCrossAdapter<M>
where
    M: MemPool + 'static,
{
    async fn watch_ckb_client(&self, ctx: Context) -> ProtocolResult<()> {
        Ok(())
    }

    async fn send_axon_tx(&self, ctx: Context, stx: SignedTransaction) -> ProtocolResult<()> {
        self.mempool.insert(ctx, stx).await
    }

    async fn send_ckb_tx(&self, ctx: Context) -> ProtocolResult<()> {
        Ok(())
    }
}
