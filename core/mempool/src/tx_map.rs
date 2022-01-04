use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::RwLock;
use rayon::prelude::*;

use protocol::types::{Hash, SignedTransaction, H160, U256};
use protocol::{traits::MixedTxHashes, ProtocolResult};

use crate::{queue::SenderTxQueue, MemPoolError};

pub struct TxMap {
    sender_map: DashMap<H160, SenderTxQueue>,
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
        self.hash_map.get(hash).map(|tx| tx.stx.clone())
    }

    pub fn len(&self) -> usize {
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
        if self.hash_map.len() >= self.capacity {
            return Err(MemPoolError::ReachLimit {
                pool_size: self.capacity,
            }
            .into());
        }

        let sender = stx.sender;
        let tx_hash = stx.transaction.hash;
        let tx_ptr = TxWrapper::from_pointee(stx);

        if let Some(old_tx) = self
            .sender_map
            .entry(sender)
            .or_insert_with(SenderTxQueue::new)
            .insert(Arc::clone(&tx_ptr))
        {
            self.hash_map.remove(&old_tx.stx.transaction.hash);
        }

        self.hash_map.insert(tx_hash, tx_ptr);

        Ok(())
    }

    pub fn remove_batch(&self, hashes: &[Hash]) {
        let _lock = self.lock.write();

        hashes.into_par_iter().for_each(|hash| {
            if let Some(res) = self.hash_map.remove(hash) {
                let sender = &res.1.stx.sender;

                let residue = self
                    .sender_map
                    .get_mut(sender)
                    .map(|mut queue| {
                        queue
                            .value_mut()
                            .remove(&res.1.stx.transaction.unsigned.nonce);

                        queue.len()
                    })
                    .unwrap_or_default();

                if residue == 0 {
                    self.sender_map.remove(sender);
                }
            }
        });
    }

    pub fn package(&self, _gas_limit: u64, tx_num_limit: u64) -> MixedTxHashes {
        let tx_num_limit = tx_num_limit as usize;
        let mut order_hashes = Vec::with_capacity(tx_num_limit);

        let mut count = 0;
        let _lock = self.lock.write();

        loop {
            if count >= tx_num_limit {
                break;
            }

            if let Some(hash) = self.select_tx() {
                order_hashes.push(hash);
                count += 1;
            } else {
                break;
            }
        }

        MixedTxHashes {
            order_tx_hashes:   order_hashes,
            propose_tx_hashes: vec![],
        }
    }

    fn select_tx(&self) -> Option<Hash> {
        if self.sender_map.is_empty() {
            return None;
        }

        let res =
            self.sender_map
                .iter()
                .fold((Hash::default(), U256::zero()), |(hash, fee), kv| {
                    let first = kv.value().first();
                    if first.max_priority_fee_per_gas > fee {
                        (first.tx_hash, first.max_priority_fee_per_gas)
                    } else {
                        (hash, fee)
                    }
                });

        Some(res.0)
    }
}

pub type TxPtr = Arc<TxWrapper>;

pub struct TxWrapper {
    pub stx:                      SignedTransaction,
    pub tx_hash:                  Hash,
    pub max_priority_fee_per_gas: U256,
}

impl From<SignedTransaction> for TxWrapper {
    fn from(stx: SignedTransaction) -> Self {
        let max_priority_fee_per_gas = stx.transaction.unsigned.max_priority_fee_per_gas;
        let tx_hash = stx.transaction.hash;

        TxWrapper {
            stx,
            tx_hash,
            max_priority_fee_per_gas,
        }
    }
}

impl TxWrapper {
    pub fn from_pointee(stx: SignedTransaction) -> Arc<TxWrapper> {
        Arc::new(stx.into())
    }
}
