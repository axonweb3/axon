mod adapter;
mod context;
mod pool;
#[cfg(test)]
mod tests;
mod tx_wrapper;

pub use adapter::message::{
    MsgPullTxs, NewTxsHandler, PullTxsHandler, END_GOSSIP_NEW_TXS, RPC_PULL_TXS, RPC_RESP_PULL_TXS,
    RPC_RESP_PULL_TXS_SYNC,
};
pub use adapter::{AdapterError, DefaultMemPoolAdapter};

use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use futures::future::try_join_all;

use common_apm::Instant;

use protocol::traits::{Context, MemPool, MemPoolAdapter};
use protocol::types::{
    AddressSource, BlockNumber, Hash, PackedTxHashes, SignedTransaction, H160, H256, U256,
};
use protocol::{async_trait, tokio, Display, ProtocolError, ProtocolErrorKind, ProtocolResult};

use core_executor::is_call_system_script;
use core_network::NetworkContext;

use crate::{context::TxContext, pool::PriorityPool};

pub struct MemPoolImpl<Adapter> {
    pool:    PriorityPool,
    adapter: Arc<Adapter>,
}

impl<Adapter> MemPoolImpl<Adapter>
where
    Adapter: MemPoolAdapter + 'static,
{
    pub async fn new(
        pool_size: usize,
        timeout_gap: u64,
        adapter: Adapter,
        initial_txs: Vec<SignedTransaction>,
    ) -> Self {
        let mempool = MemPoolImpl {
            pool:    PriorityPool::new(pool_size, timeout_gap).await,
            adapter: Arc::new(adapter),
        };

        for tx in initial_txs.into_iter() {
            if let Err(e) = mempool.initial_insert(Context::new(), tx).await {
                log::warn!("[mempool]: initial insert tx failed {:?}", e);
            }
        }

        mempool
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }

    pub fn co_queue_len(&self) -> usize {
        self.pool.co_queue_len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_adapter(&self) -> &Adapter {
        &self.adapter
    }

    async fn show_unknown_txs(&self, tx_hashes: &[Hash]) -> Vec<Hash> {
        tx_hashes
            .iter()
            .filter_map(|hash| {
                if self.pool.contains(hash) {
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
        self.pool.insert(stx, true, U256::zero())
    }

    async fn insert_tx(
        &self,
        ctx: Context,
        tx: SignedTransaction,
        is_system_script: bool,
    ) -> ProtocolResult<()> {
        let tx_hash = &tx.transaction.hash;
        if let Err(i) = self.pool.reach_limit() {
            return Err(MemPoolError::ReachLimit(i).into());
        }

        if self.pool.contains(tx_hash) {
            return Ok(());
        } else {
            let check_nonce = self.adapter.check_authorization(ctx.clone(), &tx).await?;
            self.adapter.check_transaction(ctx.clone(), &tx).await?;
            self.adapter
                .check_storage_exist(ctx.clone(), tx_hash)
                .await?;

            if is_system_script {
                self.pool.insert_system_script_tx(tx.clone())?;
            } else {
                self.pool.insert(tx.clone(), true, check_nonce)?;
            }

            if !ctx.is_network_origin_txs() {
                self.adapter.broadcast_tx(ctx, None, tx).await?;
            } else {
                let origin = ctx.session_id().unwrap();
                self.adapter
                    .broadcast_tx(ctx.clone(), Some(origin.value()), tx)
                    .await?;
                self.adapter.report_good(ctx);
            }
        }

        Ok(())
    }

    async fn verify_tx_in_parallel(
        &self,
        ctx: Context,
        txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<Vec<U256>> {
        let inst = Instant::now();
        let len = txs.len();

        let futs = txs
            .into_iter()
            .map(|tx| {
                let adapter = Arc::clone(&self.adapter);
                let ctx = ctx.clone();

                tokio::spawn(async move {
                    let check_nonce = adapter.check_authorization(ctx.clone(), &tx).await?;
                    adapter.check_transaction(ctx.clone(), &tx).await?;
                    adapter
                        .check_storage_exist(ctx.clone(), &tx.transaction.hash)
                        .await?;
                    Ok(check_nonce)
                })
            })
            .collect::<Vec<tokio::task::JoinHandle<Result<_, ProtocolError>>>>();

        let res: Vec<U256> = try_join_all(futs)
            .await
            .map_err(|e| {
                log::error!("[mempool] verify batch txs error {:?}", e);
                MemPoolError::VerifyBatchTransactions
            })?
            .into_iter()
            .collect::<Result<Vec<U256>, ProtocolError>>()?;

        log::info!(
            "[mempool] verify txs done, size {} cost {:?}",
            len,
            inst.elapsed()
        );
        Ok(res)
    }

    pub fn get_tx_cache(&self) -> &PriorityPool {
        &self.pool
    }
}

#[async_trait]
impl<Adapter> MemPool for MemPoolImpl<Adapter>
where
    Adapter: MemPoolAdapter + 'static,
{
    async fn insert(&self, ctx: Context, tx: SignedTransaction) -> ProtocolResult<()> {
        let is_call_system_script = is_call_system_script(tx.transaction.unsigned.action());

        log::debug!(
            "[mempool]: is call system script {:?}",
            is_call_system_script
        );

        self.insert_tx(ctx, tx, is_call_system_script).await
    }

    async fn contains(&self, _ctx: Context, tx_hash: &Hash) -> bool {
        self.pool.contains(tx_hash)
    }

    async fn package(
        &self,
        _ctx: Context,
        gas_limit: U256,
        tx_num_limit: u64,
    ) -> ProtocolResult<PackedTxHashes> {
        log::info!(
            "[core_mempool]: {:?} txs in map while package",
            self.pool.len(),
        );
        let inst = Instant::now();
        let txs = self.pool.package(gas_limit, tx_num_limit as usize);

        common_apm::metrics::mempool::MEMPOOL_PACKAGE_SIZE_VEC_STATIC
            .package
            .observe((txs.hashes.len()) as f64);
        common_apm::metrics::mempool::MEMPOOL_TIME_STATIC
            .package
            .observe(common_apm::metrics::duration_to_sec(inst.elapsed()));
        Ok(txs)
    }

    async fn flush(
        &self,
        _ctx: Context,
        tx_hashes: &[Hash],
        current_number: BlockNumber,
    ) -> ProtocolResult<()> {
        log::info!(
            "[core_mempool]: flush mempool with {:?} tx_hashes",
            tx_hashes.len(),
        );
        self.adapter.clear_nonce_cache();
        self.pool.flush(tx_hashes, current_number);
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
            if let Some(tx) = self.pool.get_by_hash(tx_hash) {
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

            let check_nonces = self.verify_tx_in_parallel(ctx.clone(), txs.clone()).await?;

            for (signed_tx, check_nonce) in txs.into_iter().zip(check_nonces.into_iter()) {
                let is_call_system_script =
                    is_call_system_script(signed_tx.transaction.unsigned.action());
                if is_call_system_script {
                    self.pool.insert_system_script_tx(signed_tx)?;
                } else {
                    self.pool.insert(signed_tx, false, check_nonce)?;
                }
            }

            self.adapter.report_good(ctx);
        }

        Ok(())
    }

    async fn get_tx_count_by_address(&self, _ctx: Context, address: H160) -> ProtocolResult<usize> {
        Ok(self.pool.get_tx_count_by_address(address))
    }

    fn get_tx_from_mem(&self, _ctx: Context, tx_hash: &Hash) -> Option<SignedTransaction> {
        self.pool.get_by_hash(tx_hash)
    }

    fn set_args(&self, context: Context, state_root: H256, gas_limit: u64, max_tx_size: u64) {
        self.adapter
            .set_args(context, state_root, gas_limit, max_tx_size);
    }
}

pub fn check_dup_order_hashes(order_tx_hashes: &[Hash]) -> ProtocolResult<()> {
    let mut dup_set = HashSet::with_capacity(order_tx_hashes.len());

    for hash in order_tx_hashes.iter() {
        if dup_set.contains(hash) {
            return Err(MemPoolError::EnsureDup(*hash).into());
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
        fmt = "Tx: {:?} exceeds balance, account balance: {:?}, gas limit: {:?}",
        tx_hash,
        account_balance,
        tx_gas_limit
    )]
    ExceedBalance {
        tx_hash:         Hash,
        account_balance: U256,
        tx_gas_limit:    U256,
    },

    #[display(fmt = "Invalid gas price {:?}", _0)]
    InvalidGasPrice(U256),

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

    #[display(fmt = "Tx: {:?} inserts failed", _0)]
    Insert(Hash),

    #[display(fmt = "Mempool reaches limit: {}", _0)]
    ReachLimit(usize),

    #[display(fmt = "Tx: {:?} exists in pool", _0)]
    Dup(Hash),

    #[display(fmt = "Pull txs, require: {}, response: {}", require, response)]
    EnsureBreak { require: usize, response: usize },

    #[display(
        fmt = "There is duplication in order transactions. duplication tx_hash {:?}",
        _0
    )]
    EnsureDup(Hash),

    #[display(fmt = "Fetch full txs, require: {}, response: {}", require, response)]
    MisMatch { require: usize, response: usize },

    #[display(fmt = "Tx inserts candidate_queue failed, len: {}", _0)]
    InsertCandidate(usize),

    #[display(fmt = "Tx: {:?} check authorization error {:?}", tx_hash, err_info)]
    CheckAuthorization { tx_hash: Hash, err_info: String },

    #[display(fmt = "Check_hash failed, expect: {:?}, get: {:?}", expect, actual)]
    CheckHash { expect: Hash, actual: Hash },

    #[display(fmt = "Tx: {:?} already commit", _0)]
    CommittedTx(Hash),

    #[display(fmt = "Tx: {:?} doesn't match our chain id", _0)]
    WrongChain(Hash),

    #[display(fmt = "Tx: {:?} timeout {}", tx_hash, timeout)]
    Timeout { tx_hash: Hash, timeout: u64 },

    #[display(fmt = "Tx: {:?} invalid timeout", _0)]
    InvalidTimeout(Hash),

    #[display(fmt = "Batch transaction validation failed")]
    VerifyBatchTransactions,

    #[display(fmt = "Encode transaction to JSON failed")]
    EncodeJson,

    #[display(fmt = "Invalid address source {:?}", _0)]
    InvalidAddressSource(AddressSource),

    #[display(fmt = "Invalid dummy input")]
    InvalidDummyInput,

    #[display(fmt = "Invalid sender, expect: {:?}, get: {:?}", expect, actual)]
    InvalidSender { expect: H160, actual: H160 },
}

impl Error for MemPoolError {}

impl From<MemPoolError> for ProtocolError {
    fn from(error: MemPoolError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Mempool, Box::new(error))
    }
}
