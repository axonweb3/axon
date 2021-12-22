use dashmap::DashMap;
use parking_lot::RwLock;

use protocol::types::{Hash, SignedTransaction, H160};
use protocol::ProtocolResult;

pub struct TxMap {
    map:  DashMap<H160, SignedTransaction>,
    lock: RwLock<()>,
}

impl TxMap {
    pub fn new(capacity: usize) -> Self {
        TxMap {
            map:  DashMap::with_capacity(capacity),
            lock: RwLock::new(()),
        }
    }

    pub fn insert(&self, stx: SignedTransaction) -> ProtocolResult<()> {
        let _lock = self.lock.read();
        let sender = stx.sender;
        if let Some(mut exist_tx) = self.map.get_mut(&sender) {
            if exist_tx.transaction.unsigned.nonce < stx.transaction.unsigned.nonce {
                *exist_tx = stx;
            }
        } else {
            self.map.insert(sender, stx);
        }

        Ok(())
    }

    pub fn remove_batch(&self, senders: &[H160]) {
        let _lock = self.lock.write();
        for sender in senders.iter() {
            self.map.remove(sender);
        }
    }

    pub fn package(&self) -> Vec<SignedTransaction> {
        let _lock = self.lock.write();
        self.map
            .iter()
            .map(|ref_kv| ref_kv.value().clone())
            .collect()
    }
}
