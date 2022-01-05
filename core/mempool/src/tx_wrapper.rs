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

pub fn get_tx_ptr(stx: &SignedTransaction) -> TxPtr {
    Arc::new(stx.into())
}
