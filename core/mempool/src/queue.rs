use std::collections::BTreeMap;

use protocol::types::U256;

use crate::tx_map::TxPtr;

#[derive(Clone)]
pub struct SenderTxQueue(BTreeMap<U256, TxPtr>);

impl SenderTxQueue {
    pub fn new() -> Self {
        SenderTxQueue(BTreeMap::new())
    }

    pub fn insert(&mut self, tx_ptr: TxPtr) -> Option<TxPtr> {
        self.0.insert(tx_ptr.stx.transaction.unsigned.nonce, tx_ptr)
    }

    pub fn remove(&mut self, nonce: &U256) -> Option<TxPtr> {
        self.0.remove(nonce)
    }

    pub fn _clear_by_nonce(&mut self, min_nonce: U256) {
        self.0.retain(|&k, _v| k > min_nonce);
    }

    pub fn first(&self) -> &TxPtr {
        self.0.first_key_value().unwrap().1
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
