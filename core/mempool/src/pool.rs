use std::collections::{BTreeMap, BinaryHeap, HashSet};
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
use protocol::ProtocolResult;

use crate::tx_wrapper::{TxPtr, TxWrapper};
use crate::MemPoolError;

pub struct PriorityPool {
    sys_tx_bucket:  BuiltInContractTxBucket,
    occupied_nonce: DashMap<H160, BTreeMap<U256, TxPtr>>,
    co_queue:       Arc<ArrayQueue<TxPtr>>,
    real_queue:     Arc<Mutex<BinaryHeap<TxPtr>>>,
    tx_map:         DashMap<Hash, SignedTransaction>,
    stock_len:      AtomicUsize,
    timeout_gap:    Mutex<BTreeMap<BlockNumber, HashSet<Hash>>>,
    timeout_config: u64,

    flush_lock: Arc<RwLock<()>>,
}

impl PriorityPool {
    pub async fn new(size: usize, timeout_config: u64) -> Self {
        let pool = PriorityPool {
            sys_tx_bucket: BuiltInContractTxBucket::new(),
            occupied_nonce: DashMap::new(),
            co_queue: Arc::new(ArrayQueue::new(size)),
            real_queue: Arc::new(Mutex::new(BinaryHeap::with_capacity(size * 2))),
            tx_map: DashMap::new(),
            stock_len: AtomicUsize::new(0),
            timeout_gap: Mutex::new(BTreeMap::new()),
            timeout_config,
            flush_lock: Arc::new(RwLock::new(())),
        };

        let co_queue = Arc::clone(&pool.co_queue);
        let real_queue = Arc::clone(&pool.real_queue);
        let flush_lock = Arc::clone(&pool.flush_lock);

        tokio::spawn(async move {
            loop {
                if !co_queue.is_empty() {
                    let _flushing = flush_lock.read();
                    let mut q = real_queue.lock();
                    let txs = pop_all_item(Arc::clone(&co_queue));
                    txs.for_each(|p_tx| q.push(p_tx));
                }

                sleep(Duration::from_millis(50)).await;
            }
        });

        pool
    }

    pub fn get_tx_count_by_address(&self, address: H160) -> usize {
        if let Some(set) = self.occupied_nonce.get(&address) {
            return set.iter().filter(|tx| !tx.1.is_dropped()).count();
        }

        0usize
    }

    pub fn insert_system_script_tx(&self, stx: SignedTransaction) -> ProtocolResult<()> {
        let _flushing = self.flush_lock.read();
        if self.sys_tx_bucket.insert(stx) {
            self.stock_len.fetch_add(1, Ordering::AcqRel);
        }
        Ok(())
    }

    pub fn insert(&self, stx: SignedTransaction, check_limit: bool) -> ProtocolResult<()> {
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
            let mut q = self.real_queue.lock();
            let txs = pop_all_item(Arc::clone(&self.co_queue));
            txs.for_each(|p_tx| q.push(p_tx));
        }

        let (ptr, tx) = TxWrapper::from(stx).into_parts();

        if self.tx_map.insert(ptr.hash, tx).is_some() {
            self.stock_len.fetch_sub(1, Ordering::AcqRel);
        } else {
            let _ = self.co_queue.push(Arc::clone(&ptr));
            self.occupy_nonce(ptr);
        }
        Ok(())
    }

    pub fn package(&self, _gas_limit: U256, limit: usize) -> PackedTxHashes {
        let _flushing = self.flush_lock.read();

        let mut hashes = self.sys_tx_bucket.package();
        let call_system_script_count = hashes.len() as u32;

        let mut q = self.real_queue.lock();
        if !self.co_queue.is_empty() {
            let txs = pop_all_item(Arc::clone(&self.co_queue));
            txs.for_each(|p_tx| q.push(p_tx));
        }

        hashes.extend(
            q.iter()
                .filter_map(|ptr| {
                    if ptr.is_dropped() {
                        None
                    } else {
                        Some(ptr.hash)
                    }
                })
                .take(limit),
        );

        PackedTxHashes {
            hashes,
            call_system_script_count,
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

        match self.tx_map.get(hash).map(|r| r.clone()) {
            Some(tx) => Some(tx),
            None => self.sys_tx_bucket.get_tx_by_hash(hash),
        }
    }

    pub fn flush<F: Fn(&SignedTransaction) -> bool>(
        &self,
        hashes: &[Hash],
        nonce_check: F,
        number: BlockNumber,
    ) {
        let _flushing = self.flush_lock.write();
        self.occupied_nonce.clear();
        let mut reduce_len = 0;
        self.flush_inner(hashes, nonce_check, &mut reduce_len, number);
        self.sys_tx_bucket.flush(hashes, &mut reduce_len);

        if reduce_len != 0 {
            self.stock_len.fetch_sub(reduce_len, Ordering::AcqRel);
        }
    }

    fn flush_inner<F: Fn(&SignedTransaction) -> bool>(
        &self,
        hashes: &[Hash],
        nonce_check: F,
        reduce_len: &mut usize,
        number: BlockNumber,
    ) {
        let mut q = self.real_queue.lock();
        let mut timeout_gap = self.timeout_gap.lock();

        for hash in hashes {
            if self.tx_map.remove(hash).is_some() {
                *reduce_len += 1;
            }
        }

        for tx_ptr in q.drain().chain(pop_all_item(Arc::clone(&self.co_queue))) {
            if tx_ptr.is_dropped() && self.tx_map.remove(tx_ptr.hash()).is_some() {
                *reduce_len += 1;
            }
        }

        let timeout = if number > self.timeout_config {
            timeout_gap.remove(&number.saturating_sub(self.timeout_config))
        } else {
            None
        };

        self.tx_map.retain(|_, v| {
            if nonce_check(v)
                && timeout
                    .as_ref()
                    .map(|set| set.is_empty() || !set.contains(&v.transaction.hash))
                    .unwrap_or(true)
            {
                return true;
            }
            *reduce_len += 1;
            false
        });

        let ptr = timeout_gap.entry(number).or_default();
        for tx in self.tx_map.iter().map(|kv| kv.value().clone()) {
            let tx_wrapper = TxWrapper::from(tx);
            self.occupy_nonce(tx_wrapper.ptr());
            q.push(tx_wrapper.ptr());
            ptr.insert(tx_wrapper.hash());
        }
    }

    fn occupy_nonce(&self, tx_ptr: TxPtr) {
        if let Some(old_ptr) = self
            .occupied_nonce
            .entry(tx_ptr.sender)
            .or_insert_with(BTreeMap::new)
            .insert(tx_ptr.nonce, tx_ptr)
        {
            old_ptr.set_dropped();
        }
    }

    #[cfg(test)]
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
        let data = stx.transaction.unsigned.data();
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
                    .first_key_value()
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
