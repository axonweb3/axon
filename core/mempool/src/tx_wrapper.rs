use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd, Reverse};
use std::collections::{btree_map::Entry, BTreeMap};
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::atomic::{AtomicU8, Ordering as AtomicOrdering};
use std::sync::Arc;

use protocol::types::{Hash, SignedTransaction, H160, U256};

pub type TxPtr = Arc<TxWrapper>;

#[derive(Debug)]
pub struct TxWrapper {
    // 0x00 init
    // 0x01 package
    // 0x10 drop
    state: AtomicU8,
    tx:    SignedTransaction,
}

impl From<SignedTransaction> for TxWrapper {
    fn from(stx: SignedTransaction) -> Self {
        TxWrapper {
            tx:    stx,
            state: AtomicU8::new(0),
        }
    }
}

impl Ord for TxWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.sender() != other.sender() {
            return Reverse(self.gas_price()).cmp(&Reverse(other.gas_price()));
        }
        Reverse(self.nonce()).cmp(&Reverse(other.nonce()))
    }
}

impl PartialEq for TxWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
}

impl Eq for TxWrapper {}

impl PartialOrd for TxWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TxWrapper {
    pub fn hash(&self) -> Hash {
        self.tx.transaction.hash
    }

    pub fn nonce(&self) -> &U256 {
        self.tx.transaction.unsigned.nonce()
    }

    pub fn sender(&self) -> H160 {
        self.tx.sender
    }

    pub fn gas_price(&self) -> U256 {
        self.tx.transaction.unsigned.gas_price()
    }

    pub fn raw_tx(&self) -> SignedTransaction {
        self.tx.clone()
    }

    pub fn is_dropped(&self) -> bool {
        self.state.load(AtomicOrdering::Acquire) & 0x10 == 0x10
    }

    pub fn set_dropped(&self) {
        self.state.fetch_or(0x10, AtomicOrdering::AcqRel);
    }

    fn set_package(&self) {
        self.state.fetch_or(0x01, AtomicOrdering::AcqRel);
    }

    fn is_package(&self) -> bool {
        self.state.load(AtomicOrdering::Acquire) & 0x01 == 0x01
    }
}

#[derive(Default)]
pub struct PendingQueue {
    queue:         BTreeMap<U256, TxPtr>,
    // already insert to package list tip nonce
    pop_tip_nonce: U256,
}

impl PendingQueue {
    pub fn insert(&mut self, tx: TxPtr, nonce_diff: U256) -> bool {
        let nonce = *tx.nonce();
        let current_tip = nonce - nonce_diff;
        if self.pop_tip_nonce < current_tip {
            self.pop_tip_nonce = current_tip;
        }
        match self.queue.entry(nonce) {
            Entry::Occupied(mut o) => {
                if o.get().gas_price() < tx.gas_price() {
                    let old = o.insert(Arc::clone(&tx));
                    old.set_dropped();
                    // replace with package list tx
                    if old.is_package() {
                        tx.set_package();
                        return true;
                    }
                } else {
                    tx.set_dropped();
                }
            }
            Entry::Vacant(v) => {
                v.insert(tx);
            }
        }

        false
    }

    pub fn try_search_package_list(&mut self) -> Vec<TxPtr> {
        let mut res = Vec::new();
        let mut current = self.pop_tip_nonce;
        for (k, v) in self.queue.range((Excluded(current), Unbounded)) {
            if k == &(current + 1) {
                current = current + 1;
                if v.is_package() {
                    continue;
                }
                v.set_package();
                res.push(Arc::clone(v));
            } else {
                break;
            }
        }
        self.pop_tip_nonce = current;
        res
    }

    pub fn clear_droped(&mut self) {
        self.queue.retain(|_, v| !v.is_dropped());

        self.pop_tip_nonce = U256::zero();
    }

    pub fn count(&self) -> usize {
        self.queue.values().filter(|tx| !tx.is_dropped()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}
