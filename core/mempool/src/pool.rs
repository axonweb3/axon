use std::collections::{hash_map::Entry, BTreeMap, HashMap, HashSet};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

use crossbeam_queue::ArrayQueue;
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};

use protocol::tokio::{self, time::sleep};
use protocol::types::{BlockNumber, Bytes, Hash, PackedTxHashes, SignedTransaction, H160, U256};
use protocol::{ProtocolResult, MEMPOOL_REFRESH_TIMEOUT};

use crate::tx_wrapper::{PendingQueue, TxPtr, TxWrapper};
use crate::MemPoolError;

pub struct PriorityPool {
    sys_tx_bucket:          BuiltInContractTxBucket,
    // The transaction data in this queue should be consistent with the real queue.
    // The difference is that it will be grouped by Sender and sorted by nonce to implement
    // the replace by fee function.
    pending_queue:          Arc<DashMap<H160, PendingQueue>>,
    // The transactions in this queue have not been processed yet and cannot be packaged.
    co_queue:               Arc<ArrayQueue<(TxPtr, U256)>>,
    // Transactions in this queue will be packaged into blocks
    real_queue:             Arc<Mutex<Vec<TxPtr>>>,
    // Record all transactions in the transaction pool
    tx_map:                 DashMap<Hash, TxPtr>,
    stock_len:              AtomicUsize,
    // Record the height at which the transaction first enters the transaction pool
    // There must be a record for each block height, keeping timeout_config
    pub(crate) timeout_gap: Mutex<BTreeMap<BlockNumber, HashSet<Hash>>>,
    // When a transaction is not submitted for more than timeout_config blocks
    // in the transaction pool, the transaction will be discarded.
    timeout_config:         u64,

    flush_lock: Arc<RwLock<()>>,
}

impl PriorityPool {
    pub async fn new(size: usize, timeout_config: u64) -> Self {
        let pool = PriorityPool {
            sys_tx_bucket: BuiltInContractTxBucket::new(),
            pending_queue: Arc::new(DashMap::new()),
            co_queue: Arc::new(ArrayQueue::new(size)),
            real_queue: Arc::new(Mutex::new(Vec::with_capacity(size * 2))),
            tx_map: DashMap::new(),
            stock_len: AtomicUsize::new(0),
            timeout_gap: Mutex::new(BTreeMap::new()),
            timeout_config,
            flush_lock: Arc::new(RwLock::new(())),
        };

        let co_queue = Arc::clone(&pool.co_queue);
        let real_queue = Arc::clone(&pool.real_queue);
        let pending_queues = Arc::clone(&pool.pending_queue);
        let flush_lock = Arc::clone(&pool.flush_lock);

        tokio::spawn(async move {
            loop {
                if !co_queue.is_empty() {
                    let _flushing = flush_lock.read();
                    let mut q = real_queue.lock();
                    let txs = pop_all_item(Arc::clone(&co_queue));
                    for (tx, nonce_diff) in txs {
                        let mut pending_queue = pending_queues.entry(tx.sender()).or_default();

                        // drop this tx
                        if pending_queue.len() > 64 {
                            tx.set_dropped();
                            continue;
                        }

                        // replace with real queue tx
                        if pending_queue.insert(Arc::clone(&tx), nonce_diff) {
                            q.push(tx);
                        }

                        pending_queue.try_search_package_list(&mut q);
                    }
                }

                sleep(Duration::from_millis(MEMPOOL_REFRESH_TIMEOUT)).await;
            }
        });

        pool
    }

    pub fn get_tx_count_by_address(&self, address: H160) -> (usize, Option<BlockNumber>) {
        let number = self.timeout_gap.lock().last_entry().map(|f| *f.key());
        if let Some(set) = self.pending_queue.get(&address) {
            return (set.count(), number);
        }

        (0usize, number)
    }

    pub fn insert_system_script_tx(&self, stx: SignedTransaction) -> ProtocolResult<()> {
        let _flushing = self.flush_lock.read();
        if self.sys_tx_bucket.insert(stx) {
            self.stock_len.fetch_add(1, Ordering::AcqRel);
        }
        Ok(())
    }

    pub fn insert(
        &self,
        stx: SignedTransaction,
        check_limit: bool,
        check_nonce: U256,
    ) -> ProtocolResult<()> {
        if let Err(n) = self
            .stock_len
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |x| {
                if x >= self.co_queue.capacity() && check_limit {
                    None
                } else {
                    Some(x + 1)
                }
            })
        {
            return Err(MemPoolError::ReachLimit(n).into());
        }

        // This lock is necessary to avoid mismatch error triggered by the concurrent
        // operation of tx insertion and flush.
        let _flushing = self.flush_lock.read();

        // Must flush co_queue here when it's full, otherwise, this tx may can't package
        // by self, because it will never insert to real_queue
        if !check_limit && self.co_queue.is_full() {
            self.flush_to_pending_queue()
        }

        let ptr = Arc::new(TxWrapper::from(stx));

        match self.tx_map.entry(ptr.hash()) {
            dashmap::mapref::entry::Entry::Occupied(_) => {
                self.stock_len.fetch_sub(1, Ordering::AcqRel);
            }
            dashmap::mapref::entry::Entry::Vacant(v) => {
                v.insert(Arc::clone(&ptr));
                let _ = self.co_queue.push((ptr, check_nonce));
            }
        }

        Ok(())
    }

    pub fn package(&self, _gas_limit: U256, limit: usize) -> PackedTxHashes {
        let _flushing = self.flush_lock.read();

        let mut hashes = self.sys_tx_bucket.package();
        let call_system_script_count = hashes.len() as u32;

        if !self.co_queue.is_empty() {
            self.flush_to_pending_queue()
        }
        let mut q = self.real_queue.lock();

        q.sort_unstable();

        hashes.extend(
            q.iter()
                .filter_map(|ptr| {
                    if ptr.is_dropped() {
                        None
                    } else {
                        Some(ptr.hash())
                    }
                })
                .take(limit),
        );

        PackedTxHashes {
            hashes,
            call_system_script_count,
        }
    }

    fn flush_to_pending_queue(&self) {
        let mut q = self.real_queue.lock();
        let txs = pop_all_item(Arc::clone(&self.co_queue));
        for (tx, nonce_diff) in txs {
            let mut pending_queue = self.pending_queue.entry(tx.sender()).or_default();

            // drop this tx
            if pending_queue.len() > 64 {
                tx.set_dropped();
                continue;
            }

            // replace with real queue tx
            if pending_queue.insert(Arc::clone(&tx), nonce_diff) {
                q.push(tx);
            }

            pending_queue.try_search_package_list(&mut q);
        }
    }

    pub fn len(&self) -> usize {
        self.stock_len.load(Ordering::Acquire)
    }

    pub fn co_queue_len(&self) -> usize {
        self.co_queue.len()
    }

    pub fn contains(&self, hash: &Hash) -> bool {
        let _flushing = self.flush_lock.read();
        self.tx_map.contains_key(hash) || self.sys_tx_bucket.contains(hash)
    }

    pub fn reach_limit(&self) -> Result<usize, usize> {
        let c = self.len();
        if c > self.co_queue.capacity() {
            Err(c)
        } else {
            Ok(c)
        }
    }

    pub fn get_by_hash(&self, hash: &Hash) -> Option<SignedTransaction> {
        let _flushing = self.flush_lock.read();

        match self.tx_map.get(hash).map(|r| r.raw_tx()) {
            Some(tx) => Some(tx),
            None => self.sys_tx_bucket.get_tx_by_hash(hash),
        }
    }

    pub fn flush(&self, hashes: &[Hash], number: BlockNumber) {
        let _flushing = self.flush_lock.write();
        self.flush_to_pending_queue();
        let mut reduce_len = 0;
        self.flush_inner(hashes, &mut reduce_len, number);
        self.sys_tx_bucket.flush(hashes, &mut reduce_len);

        if reduce_len != 0 {
            self.stock_len.fetch_sub(reduce_len, Ordering::AcqRel);
        }
    }

    fn flush_inner(&self, hashes: &[Hash], reduce_len: &mut usize, number: BlockNumber) {
        let mut q = self.real_queue.lock();
        let mut timeout_gap = self.timeout_gap.lock();

        let mut remove_tip_nonce: HashMap<H160, U256> = HashMap::new();
        for hash in hashes {
            if let Some((_, ptr)) = self.tx_map.remove(hash) {
                match remove_tip_nonce.entry(ptr.sender()) {
                    Entry::Occupied(mut v) => {
                        if v.get() < ptr.nonce() {
                            v.insert(*ptr.nonce());
                        }
                    }
                    Entry::Vacant(v) => {
                        v.insert(*ptr.nonce());
                    }
                }
                *reduce_len += 1;
            }
        }

        for (k, v) in remove_tip_nonce {
            if let Some(mut value) = self.pending_queue.get_mut(&k) {
                value.value_mut().set_drop_by_nonce_tip(v);
            }
        }

        let timeout = if number >= self.timeout_config {
            timeout_gap
                .remove(&number.saturating_sub(self.timeout_config))
                .unwrap_or_default()
        } else {
            HashSet::new()
        };

        let mut retain_keys = Vec::with_capacity(self.len() / 4);
        self.tx_map.retain(|hash, v| {
            if !v.is_dropped() && (timeout.is_empty() || !timeout.contains(&v.hash())) {
                retain_keys.push(*hash);
                return true;
            }

            v.set_dropped();
            *reduce_len += 1;
            false
        });

        timeout_gap.entry(number).or_default().extend(retain_keys);

        q.retain(|ptr| !ptr.is_dropped());

        self.pending_queue.retain(|_, v| {
            v.clear_droped();
            !v.need_remove()
        })
    }

    pub fn real_queue_len(&self) -> usize {
        self.real_queue.lock().len()
    }

    #[cfg(test)]
    pub fn system_script_queue_len(&self) -> usize {
        self.sys_tx_bucket.len()
    }
}

struct BuiltInContractTxBucket {
    hash_data_map: DashMap<Hash, Bytes>,
    tx_buckets:    DashMap<Bytes, BTreeMap<Hash, SignedTransaction>>,
}

impl BuiltInContractTxBucket {
    pub fn new() -> Self {
        BuiltInContractTxBucket {
            hash_data_map: DashMap::new(),
            tx_buckets:    DashMap::new(),
        }
    }

    pub fn insert(&self, stx: SignedTransaction) -> bool {
        let data: Bytes = stx.transaction.unsigned.data().to_vec().into();
        self.hash_data_map
            .insert(stx.transaction.hash, data.clone());
        self.tx_buckets
            .entry(data)
            .or_insert_with(BTreeMap::new)
            .insert(stx.transaction.hash, stx)
            .is_none()
    }

    pub fn package(&self) -> Vec<Hash> {
        self.tx_buckets
            .iter()
            .map(|kv| {
                kv.value()
                    .iter()
                    .take(1)
                    .next()
                    .map(|(hash, _tx)| *hash)
                    .unwrap()
            })
            .collect()
    }

    pub fn get_tx_by_hash(&self, hash: &Hash) -> Option<SignedTransaction> {
        if let Some(data) = self.hash_data_map.get(hash) {
            if let Some(tx_map) = self.tx_buckets.get(data.value()) {
                return tx_map.value().get(hash).cloned();
            }
        }

        None
    }

    pub fn flush(&self, hashes: &[Hash], reduce_len: &mut usize) {
        for hash in hashes.iter() {
            if let Some(data) = self.hash_data_map.remove(hash) {
                if self.tx_buckets.remove(&data.1).is_some() {
                    *reduce_len += 1;
                }
            }
        }
    }

    pub fn contains(&self, hash: &Hash) -> bool {
        if let Some(data) = self.hash_data_map.get(hash) {
            if let Some(tx_map) = self.tx_buckets.get(data.value()) {
                return tx_map.contains_key(hash);
            }
        }

        false
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.tx_buckets.len()
    }
}

fn pop_all_item<T>(queue: Arc<ArrayQueue<T>>) -> impl Iterator<Item = T> {
    (0..queue.len()).map(move |_| queue.pop().unwrap())
}
