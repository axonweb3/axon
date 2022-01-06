use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::Arc;

use protocol::types::{Hash, SignedTransaction, H160, U256};

pub type TxPtr = Arc<TxDigest>;

#[derive(Clone, Debug)]
pub struct TxWrapper(TxPtr, SignedTransaction);

impl From<SignedTransaction> for TxWrapper {
    fn from(stx: SignedTransaction) -> Self {
        TxWrapper(get_tx_ptr(&stx), stx)
    }
}

impl TxWrapper {
    pub fn ptr(&self) -> TxPtr {
        Arc::clone(&self.0)
    }

    pub fn hash(&self) -> Hash {
        self.0.hash
    }

    pub fn into_signed_transaction(self) -> SignedTransaction {
        self.1
    }
}

#[derive(Debug)]
pub struct TxDigest {
    pub hash:      Hash,
    pub gas_price: U256,
    pub nonce:     U256,
    pub sender:    H160,

    pub is_dropped: AtomicBool,
}

impl PartialEq for TxDigest {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for TxDigest {}

impl PartialOrd for TxDigest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TxDigest {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.gas_price != other.gas_price {
            return self.gas_price.cmp(&other.gas_price);
        }

        match self.nonce.cmp(&other.nonce) {
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => Ordering::Equal,
        }
    }
}

impl From<&SignedTransaction> for TxDigest {
    fn from(stx: &SignedTransaction) -> Self {
        TxDigest {
            hash:       stx.transaction.hash,
            gas_price:  stx.transaction.unsigned.gas_price,
            nonce:      stx.transaction.unsigned.nonce,
            sender:     stx.sender,
            is_dropped: AtomicBool::new(false),
        }
    }
}

impl TxDigest {
    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    pub fn is_dropped(&self) -> bool {
        self.is_dropped.load(AtomicOrdering::Relaxed)
    }

    pub fn set_dropped(&self) {
        self.is_dropped.swap(true, AtomicOrdering::Acquire);
    }
}

fn get_tx_ptr(stx: &SignedTransaction) -> TxPtr {
    Arc::new(stx.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BinaryHeap;

    use rand::random;

    fn rand_hash() -> Hash {
        Hash::from_slice(&(0..32).map(|_| random::<u8>()).collect::<Vec<_>>())
    }

    fn mock_tx_digest(gas_price: u64, nonce: u64) -> Arc<TxDigest> {
        Arc::new(TxDigest {
            hash:       rand_hash(),
            gas_price:  gas_price.into(),
            nonce:      nonce.into(),
            sender:     H160::default(),
            is_dropped: AtomicBool::new(false),
        })
    }

    #[test]
    fn test_tx_digest_sort() {
        let tx_1 = mock_tx_digest(1, 10);
        let tx_2 = mock_tx_digest(3, 15);
        let tx_3 = mock_tx_digest(2, 5);
        let tx_4 = mock_tx_digest(2, 3);

        let mut heap = BinaryHeap::new();
        heap.push(Arc::clone(&tx_1));
        heap.push(Arc::clone(&tx_3));
        heap.push(Arc::clone(&tx_2));
        heap.push(Arc::clone(&tx_4));

        assert_eq!(heap.pop().unwrap(), tx_2);
        assert_eq!(heap.pop().unwrap(), tx_4);
        assert_eq!(heap.pop().unwrap(), tx_3);
        assert_eq!(heap.pop().unwrap(), tx_1);
    }
}
