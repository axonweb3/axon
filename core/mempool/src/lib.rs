#![feature(test, map_first_last)]
#![allow(clippy::suspicious_else_formatting, clippy::mutable_key_type)]

mod adapter;
mod context;
mod queue;
#[cfg(test)]
mod tests;
mod tx_map;

pub use adapter::message::{
    MsgNewTxs, MsgPullTxs, MsgPushTxs, NewTxsHandler, PullTxsHandler, END_GOSSIP_NEW_TXS,
    RPC_PULL_TXS, RPC_RESP_PULL_TXS, RPC_RESP_PULL_TXS_SYNC,
};
pub use adapter::DefaultMemPoolAdapter;
pub use adapter::{DEFAULT_BROADCAST_TXS_INTERVAL, DEFAULT_BROADCAST_TXS_SIZE};

use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

use futures::future::try_join_all;

use protocol::traits::{Context, MemPool, MemPoolAdapter, MixedTxHashes};
use protocol::types::{Hash, SignedTransaction, H256};
use protocol::{async_trait, tokio, Display, ProtocolError, ProtocolErrorKind, ProtocolResult};

use crate::context::TxContext;
use crate::tx_map::TxMap;

pub struct HashMemPool<Adapter> {
    map:     TxMap,
    adapter: Arc<Adapter>,
}

impl<Adapter> HashMemPool<Adapter>
where
    Adapter: MemPoolAdapter + 'static,
{
    pub async fn new(
        pool_size: usize,
        adapter: Adapter,
        initial_txs: Vec<SignedTransaction>,
    ) -> Self {
        let mempool = HashMemPool {
            map:     TxMap::new(pool_size),
            adapter: Arc::new(adapter),
        };

        for tx in initial_txs.into_iter() {
            if let Err(e) = mempool.initial_insert(Context::new(), tx).await {
                log::warn!("[mempool]: initial insert tx failed {:?}", e);
            }
        }

        mempool
    }

    pub fn map_len(&self) -> usize {
        self.map.len()
    }

    pub fn get_adapter(&self) -> &Adapter {
        &self.adapter
    }

    async fn show_unknown_txs(&self, tx_hashes: &[Hash]) -> Vec<Hash> {
        tx_hashes
            .iter()
            .filter_map(|hash| {
                if self.map.contains(hash) {
                    None
                } else {
                    Some(*hash)
                }
            })
            .collect()
    }

    async fn initial_insert(&self, ctx: Context, stx: SignedTransaction) -> ProtocolResult<()> {
        self.adapter
            .check_storage_exist(ctx.clone(), &stx.transaction.hash)
            .await?;
        self.map.insert(stx)
    }

    async fn insert_tx(&self, ctx: Context, tx: SignedTransaction) -> ProtocolResult<()> {
        let tx = Box::new(tx);
        let tx_hash = &tx.transaction.hash;
        if self.map.reach_limit() {
            return Err(MemPoolError::ReachLimit {
                pool_size: self.map.pool_size(),
            }
            .into());
        }

        self.adapter
            .check_authorization(ctx.clone(), tx.clone())
            .await?;
        self.adapter.check_transaction(ctx.clone(), &tx).await?;
        self.adapter
            .check_storage_exist(ctx.clone(), tx_hash)
            .await?;

        self.map.insert(*tx.clone())?;

        if !ctx.is_network_origin_txs() {
            self.adapter.broadcast_tx(ctx, *tx).await?;
        } else {
            self.adapter.report_good(ctx);
        }

        Ok(())
    }

    async fn verify_tx_in_parallel(&self, ctx: Context, tx_ptrs: Vec<usize>) -> ProtocolResult<()> {
        let now = Instant::now();
        let len = tx_ptrs.len();

        let futs = tx_ptrs
            .into_iter()
            .map(|ptr| {
                let adapter = Arc::clone(&self.adapter);
                let ctx = ctx.clone();

                tokio::spawn(async move {
                    let boxed_stx = unsafe { Box::from_raw(ptr as *mut SignedTransaction) };
                    let signed_tx = *(boxed_stx.clone());

                    adapter.check_authorization(ctx.clone(), boxed_stx).await?;
                    adapter.check_transaction(ctx.clone(), &signed_tx).await?;
                    adapter
                        .check_storage_exist(ctx.clone(), &signed_tx.transaction.hash)
                        .await
                })
            })
            .collect::<Vec<_>>();

        try_join_all(futs).await.map_err(|e| {
            log::error!("[mempool] verify batch txs error {:?}", e);
            MemPoolError::VerifyBatchTransactions
        })?;

        log::info!(
            "[mempool] verify txs done, size {:?} cost {:?}",
            len,
            now.elapsed()
        );
        Ok(())
    }

    #[cfg(test)]
    pub fn get_tx_cache(&self) -> &TxMap {
        &self.map
    }
}

#[async_trait]
impl<Adapter> MemPool for HashMemPool<Adapter>
where
    Adapter: MemPoolAdapter + 'static,
{
    async fn insert(&self, ctx: Context, tx: SignedTransaction) -> ProtocolResult<()> {
        self.insert_tx(ctx, tx).await
    }

    async fn package(
        &self,
        _ctx: Context,
        gas_limit: u64,
        tx_num_limit: u64,
    ) -> ProtocolResult<MixedTxHashes> {
        log::info!(
            "[core_mempool]: {:?} txs in map while package",
            self.map.len(),
        );
        let inst = Instant::now();
        let txs = self.map.package(gas_limit, tx_num_limit);

        common_apm::metrics::mempool::MEMPOOL_PACKAGE_SIZE_VEC_STATIC
            .package
            .observe((txs.order_tx_hashes.len()) as f64);
        common_apm::metrics::mempool::MEMPOOL_TIME_STATIC
            .package
            .observe(common_apm::metrics::duration_to_sec(inst.elapsed()));
        Ok(txs)
    }

    async fn flush(&self, _ctx: Context, tx_hashes: &[Hash]) -> ProtocolResult<()> {
        log::info!(
            "[core_mempool]: flush mempool with {:?} tx_hashes",
            tx_hashes.len(),
        );
        self.map.remove_batch(tx_hashes);

        Ok(())
    }

    // This method is used to handle fetch signed transactions rpc request from
    // other nodes.
    async fn get_full_txs(
        &self,
        ctx: Context,
        height: Option<u64>,
        tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<SignedTransaction>> {
        let len = tx_hashes.len();
        let mut missing_hashes = vec![];
        let mut full_txs = Vec::with_capacity(len);

        for tx_hash in tx_hashes.iter() {
            if let Some(tx) = self.map.get_by_hash(tx_hash) {
                full_txs.push(tx);
            } else {
                missing_hashes.push(*tx_hash);
            }
        }

        // for push txs when local mempool is flushed, but the remote node still fetch
        // full block
        if !missing_hashes.is_empty() {
            full_txs.extend(
                self.adapter
                    .get_transactions_from_storage(ctx, height, &missing_hashes)
                    .await?
                    .into_iter()
                    .flatten(),
            );
        }

        if full_txs.len() != len {
            Err(MemPoolError::MisMatch {
                require:  len,
                response: full_txs.len(),
            }
            .into())
        } else {
            Ok(full_txs)
        }
    }

    async fn ensure_order_txs(
        &self,
        ctx: Context,
        height: Option<u64>,
        order_tx_hashes: &[Hash],
    ) -> ProtocolResult<()> {
        check_dup_order_hashes(order_tx_hashes)?;

        let unknown_hashes = self.show_unknown_txs(order_tx_hashes).await;
        if !unknown_hashes.is_empty() {
            let unknown_len = unknown_hashes.len();
            let txs = self
                .adapter
                .pull_txs(ctx.clone(), height, unknown_hashes)
                .await?;

            // Make sure response signed_txs is the same size of request hashes.
            if txs.len() != unknown_len {
                return Err(MemPoolError::EnsureBreak {
                    require:  unknown_len,
                    response: txs.len(),
                }
                .into());
            }

            let (tx_ptrs, txs): (Vec<_>, Vec<_>) = txs
                .into_iter()
                .map(|tx| {
                    let boxed = Box::new(tx);
                    (Box::into_raw(boxed.clone()) as usize, boxed)
                })
                .unzip();

            self.verify_tx_in_parallel(ctx.clone(), tx_ptrs).await?;

            for signed_tx in txs.into_iter() {
                self.map.insert(*signed_tx)?;
            }

            self.adapter.report_good(ctx);
        }

        Ok(())
    }

    async fn sync_propose_txs(
        &self,
        _ctx: Context,
        _propose_tx_hashes: Vec<Hash>,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    fn set_args(
        &self,
        context: Context,
        state_root: H256,
        timeout_gap: u64,
        gas_limit: u64,
        max_tx_size: u64,
    ) {
        self.adapter
            .set_args(context, state_root, timeout_gap, gas_limit, max_tx_size);
    }
}

fn check_dup_order_hashes(order_tx_hashes: &[Hash]) -> ProtocolResult<()> {
    let mut dup_set = HashSet::with_capacity(order_tx_hashes.len());

    for hash in order_tx_hashes.iter() {
        if dup_set.contains(hash) {
            return Err(MemPoolError::EnsureDup { hash: *hash }.into());
        }

        dup_set.insert(hash);
    }

    Ok(())
}

pub enum TxType {
    NewTx,
    ProposeTx,
}

// Todo: change the error.
#[derive(Debug, Display)]
pub enum MemPoolError {
    #[display(
        fmt = "Tx: {:?} exceeds size limit, now: {}, limit: {} Bytes",
        tx_hash,
        size,
        max_tx_size
    )]
    ExceedSizeLimit {
        tx_hash:     Hash,
        max_tx_size: usize,
        size:        usize,
    },

    #[display(
        fmt = "Tx: {:?} exceeds cycle limit, tx: {}, config: {}",
        tx_hash,
        gas_limit_tx,
        gas_limit_config
    )]
    ExceedGasLimit {
        tx_hash:          Hash,
        gas_limit_config: u64,
        gas_limit_tx:     u64,
    },

    #[display(fmt = "Tx nonce {} is invalid current nonce {}", tx_nonce, current)]
    InvalidNonce { current: u64, tx_nonce: u64 },

    #[display(fmt = "Tx: {:?} inserts failed", tx_hash)]
    Insert { tx_hash: Hash },

    #[display(fmt = "Mempool reaches limit: {}", pool_size)]
    ReachLimit { pool_size: usize },

    #[display(fmt = "Tx: {:?} exists in pool", tx_hash)]
    Dup { tx_hash: Hash },

    #[display(fmt = "Pull txs, require: {}, response: {}", require, response)]
    EnsureBreak { require: usize, response: usize },

    #[display(
        fmt = "There is duplication in order transactions. duplication tx_hash {:?}",
        hash
    )]
    EnsureDup { hash: Hash },

    #[display(fmt = "Fetch full txs, require: {}, response: {}", require, response)]
    MisMatch { require: usize, response: usize },

    #[display(fmt = "Tx inserts candidate_queue failed, len: {}", len)]
    InsertCandidate { len: usize },

    #[display(fmt = "Tx: {:?} check authorization error {:?}", tx_hash, err_info)]
    CheckAuthorization { tx_hash: Hash, err_info: String },

    #[display(fmt = "Check_hash failed, expect: {:?}, get: {:?}", expect, actual)]
    CheckHash { expect: Hash, actual: Hash },

    #[display(fmt = "Tx: {:?} already commit", tx_hash)]
    CommittedTx { tx_hash: Hash },

    #[display(fmt = "Tx: {:?} doesn't match our chain id", tx_hash)]
    WrongChain { tx_hash: Hash },

    #[display(fmt = "Tx: {:?} timeout {}", tx_hash, timeout)]
    Timeout { tx_hash: Hash, timeout: u64 },

    #[display(fmt = "Tx: {:?} invalid timeout", tx_hash)]
    InvalidTimeout { tx_hash: Hash },

    #[display(fmt = "Batch transaction validation failed")]
    VerifyBatchTransactions,

    #[display(fmt = "Encode transaction to JSON failed")]
    EncodeJson,
}

impl Error for MemPoolError {}

impl From<MemPoolError> for ProtocolError {
    fn from(error: MemPoolError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Mempool, Box::new(error))
    }
}
