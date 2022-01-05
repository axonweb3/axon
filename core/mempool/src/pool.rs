use std::collections::{BTreeMap, BinaryHeap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use crossbeam_queue::ArrayQueue;
use dashmap::DashMap;
use parking_lot::Mutex;

use protocol::tokio::{self, time::sleep};
use protocol::types::{Hash, SignedTransaction, H160, U256};
use protocol::ProtocolResult;

use crate::tx_wrapper::{TxPtr, TxWrapper};
use crate::MemPoolError;

pub struct PirorityPool {
    occupied_nonce: DashMap<H160, BTreeMap<U256, TxPtr>>,
    co_queue:       Arc<ArrayQueue<TxPtr>>,
    real_queue:     Arc<Mutex<BinaryHeap<TxPtr>>>,
    tx_map:         DashMap<Hash, SignedTransaction>,

    flush_lock: Arc<Mutex<()>>,
}

impl PirorityPool {
    pub async fn new(size: usize) -> Self {
        let pool = PirorityPool {
            occupied_nonce: DashMap::new(),
            co_queue:       Arc::new(ArrayQueue::new(size * 2)),
            real_queue:     Arc::new(Mutex::new(BinaryHeap::with_capacity(size))),
            tx_map:         DashMap::new(),
            flush_lock:     Arc::new(Mutex::new(())),
        };

        let co_queue = Arc::clone(&pool.co_queue);
        let real_queue = Arc::clone(&pool.real_queue);
        let flush_lock = Arc::clone(&pool.flush_lock);

        tokio::spawn(async move {
            loop {
                if !co_queue.is_empty() {
                    let _ = flush_lock.lock();
                    let txs = pop_all_item(Arc::clone(&co_queue));
                    let mut q = real_queue.lock();
                    txs.into_iter().for_each(|p_tx| q.push(p_tx));
                }

                sleep(Duration::from_millis(50)).await;
            }
        });

        pool
    }

    pub fn insert(&self, stx: SignedTransaction) -> ProtocolResult<()> {
        if self.co_queue.is_full() {
            return Err(MemPoolError::ReachLimit {
                pool_size: self.co_queue.len(),
            }
            .into());
        }

        let tx_wrapper = TxWrapper::from(stx);
        let _ = self.co_queue.push(tx_wrapper.ptr());
        self.occupy_nonce(tx_wrapper.ptr());
        self.tx_map
            .insert(tx_wrapper.hash(), tx_wrapper.into_signed_transaction());
        Ok(())
    }

    pub fn package(&self, _gas_limit: u64, limit: usize) -> Vec<Hash> {
        self.real_queue
            .lock()
            .iter()
            .take(limit)
            .map(|ptr| ptr.hash)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.tx_map.len()
    }

    pub fn contains(&self, hash: &Hash) -> bool {
        self.tx_map.contains_key(hash)
    }

    pub fn reach_limit(&self) -> bool {
        self.co_queue.is_full()
    }

    pub fn pool_size(&self) -> usize {
        self.co_queue.capacity() / 2
    }

    pub fn get_by_hash(&self, hash: &Hash) -> Option<SignedTransaction> {
        self.tx_map.get(hash).map(|r| r.clone())
    }

    pub fn flush(&self, hashes: &[Hash]) -> ProtocolResult<()> {
        let _ = self.flush_lock.lock();

        self.clear_co_queue();
        let residual = self.get_residual(hashes);

        self.clear_all();

        for tx in residual.into_iter() {
            self.insert(tx)?;
        }

        Ok(())
    }

    fn get_residual(&self, hashes: &[Hash]) -> Vec<SignedTransaction> {
        let hashes = hashes.iter().collect::<HashSet<_>>();
        let q = self.real_queue.lock();

        for tx_ptr in q.iter() {
            if hashes.contains(&tx_ptr.hash()) || tx_ptr.is_dropped() {
                self.tx_map.remove(tx_ptr.hash());
            }
        }

        self.tx_map.iter().map(|kv| kv.value().clone()).collect()
    }

    fn clear_co_queue(&self) {
        let txs = pop_all_item(Arc::clone(&self.co_queue));
        let mut q = self.real_queue.lock();
        txs.into_iter().for_each(|p_tx| q.push(p_tx));
    }

    fn clear_all(&self) {
        self.occupied_nonce.clear();
        self.tx_map.clear();
        self.real_queue.lock().clear();
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
}

fn pop_all_item<T>(queue: Arc<ArrayQueue<T>>) -> Vec<T> {
    (0..queue.len()).map(|_| queue.pop().unwrap()).collect()
}
