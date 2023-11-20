use std::sync::Arc;

use futures::future::{try_join_all, TryFutureExt};
use rlp_derive::{RlpDecodable, RlpEncodable};

use common_apm::Instant;
use protocol::{
    async_trait,
    constants::endpoints::RPC_RESP_PULL_TXS,
    tokio,
    traits::{Context, MemPool, MessageHandler, Priority, Rpc, TrustFeedback},
    types::{BatchSignedTxs, Hash, SignedTransaction},
};

use crate::context::TxContext;

pub struct NewTxsHandler<M> {
    mem_pool: Arc<M>,
}

impl<M> NewTxsHandler<M>
where
    M: MemPool,
{
    pub fn new(mem_pool: Arc<M>) -> Self {
        NewTxsHandler { mem_pool }
    }
}

#[async_trait]
impl<M> MessageHandler for NewTxsHandler<M>
where
    M: MemPool + 'static,
{
    type Message = BatchSignedTxs;

    async fn process(&self, ctx: Context, msg: Self::Message) -> TrustFeedback {
        let ctx = ctx.mark_network_origin_new_txs();

        let insert_stx = |stx: SignedTransaction| -> _ {
            let mem_pool = Arc::clone(&self.mem_pool);
            let ctx = ctx.clone();

            tokio::spawn(async move {
                let inst = Instant::now();
                common_apm::metrics::mempool::MEMPOOL_COUNTER_STATIC
                    .insert_tx_from_p2p
                    .inc();

                let res = mem_pool.insert(ctx, stx).await;

                if res.is_err() {
                    common_apm::metrics::mempool::MEMPOOL_RESULT_COUNTER_STATIC
                        .insert_tx_from_p2p
                        .failure
                        .inc();
                }
                common_apm::metrics::mempool::MEMPOOL_RESULT_COUNTER_STATIC
                    .insert_tx_from_p2p
                    .success
                    .inc();
                common_apm::metrics::mempool::MEMPOOL_TIME_STATIC
                    .insert_tx_from_p2p
                    .observe(common_apm::metrics::duration_to_sec(inst.elapsed()));
            })
        };

        // Concurrently insert them
        if try_join_all(msg.inner().into_iter().map(insert_stx).collect::<Vec<_>>())
            .await
            .map(|_| ())
            .is_err()
        {
            log::error!("[core_mempool] mempool batch insert error");
        }

        TrustFeedback::Neutral
    }
}

#[derive(Clone, Debug, Hash, RlpEncodable, RlpDecodable)]
pub struct MsgPullTxs {
    pub height: Option<u64>,
    pub hashes: Vec<Hash>,
}

pub struct PullTxsHandler<N, M> {
    network:  Arc<N>,
    mem_pool: Arc<M>,
}

impl<N, M> PullTxsHandler<N, M>
where
    N: Rpc + 'static,
    M: MemPool + 'static,
{
    pub fn new(network: Arc<N>, mem_pool: Arc<M>) -> Self {
        PullTxsHandler { network, mem_pool }
    }
}

#[async_trait]
impl<N, M> MessageHandler for PullTxsHandler<N, M>
where
    N: Rpc + 'static,
    M: MemPool + 'static,
{
    type Message = MsgPullTxs;

    async fn process(&self, ctx: Context, msg: Self::Message) -> TrustFeedback {
        let push_txs = async move {
            let ret = self
                .mem_pool
                .get_full_txs(ctx.clone(), msg.height, &msg.hashes)
                .await
                .map(BatchSignedTxs::new);

            self.network
                .response::<BatchSignedTxs>(ctx, RPC_RESP_PULL_TXS, ret, Priority::High)
                .await
        };

        push_txs
            .unwrap_or_else(move |err| log::warn!("[core_mempool] push txs {}", err))
            .await;

        TrustFeedback::Neutral
    }
}
