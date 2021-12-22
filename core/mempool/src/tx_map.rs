use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::RwLock;

use protocol::types::{Hash, SignedTransaction, H160};
use protocol::{traits::MixedTxHashes, ProtocolResult};

use crate::MemPoolError;

pub type TxPtr = Arc<SignedTransaction>;

pub struct TxMap {
    sender_map: DashMap<H160, TxPtr>,
    hash_map:   DashMap<Hash, TxPtr>,
    capacity:   usize,

    lock: RwLock<()>,
}

impl TxMap {
    pub fn new(capacity: usize) -> Self {
        TxMap {
            sender_map: DashMap::with_capacity(capacity),
            hash_map: DashMap::with_capacity(capacity),
            capacity,

            lock: RwLock::new(()),
        }
    }

    pub fn get_by_hash(&self, hash: &Hash) -> Option<SignedTransaction> {
        self.hash_map.get(hash).map(|tx| {
            let tx_ptr = Arc::clone(&tx);
            unsafe { (&*Arc::into_raw(tx_ptr)).clone() }
        })
    }

    pub fn map_len(&self) -> usize {
        self.hash_map.len()
    }

    pub fn pool_size(&self) -> usize {
        self.capacity
    }

    pub fn reach_limit(&self) -> bool {
        self.hash_map.len() >= self.capacity
    }

    pub fn contains(&self, hash: &Hash) -> bool {
        self.hash_map.contains_key(hash)
    }

    pub fn insert(&self, stx: SignedTransaction) -> ProtocolResult<()> {
        if self.contains(&stx.transaction.hash) {
            return Ok(());
        }

        let _lock = self.lock.read();
        if self.sender_map.len() == self.capacity {
            return Err(MemPoolError::ReachLimit {
                pool_size: self.capacity,
            }
            .into());
        }

        let sender = stx.sender;
        let stx = Arc::new(stx);
        if let Some(mut ref_kv) = self.sender_map.get_mut(&sender) {
            if ref_kv.value().transaction.unsigned.nonce < stx.transaction.unsigned.nonce {
                let old_tx_hash = ref_kv.value().transaction.hash;
                *ref_kv.value_mut() = stx.clone();
                self.hash_map.remove(&old_tx_hash);
                self.hash_map.insert(stx.transaction.hash, stx);
            }
        } else {
            self.sender_map.insert(stx.sender, stx.clone());
            self.hash_map.insert(stx.transaction.hash, stx);
        }

        Ok(())
    }

    pub fn remove_batch(&self, hashes: &[Hash]) {
        let _lock = self.lock.write();
        hashes.iter().for_each(|hash| {
            if let Some(tx) = self.hash_map.remove(hash) {
                self.sender_map.remove(&tx.1.sender);
            }
        });
    }

    pub fn package(&self, _gas_limit: u64, tx_num_limit: u64) -> MixedTxHashes {
        let mut order_hashes = Vec::with_capacity(tx_num_limit as usize);
        let mut count = 0;
        let _lock = self.lock.write();
        for ref_kv in self.hash_map.iter() {
            if count == tx_num_limit {
                break;
            }

            order_hashes.push(*ref_kv.key());
            count += 1;
        }

        MixedTxHashes {
            order_tx_hashes:   order_hashes,
            propose_tx_hashes: vec![],
        }
    }
}
