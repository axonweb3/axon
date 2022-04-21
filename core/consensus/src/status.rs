use std::sync::Arc;

use parking_lot::Mutex;

use protocol::types::{BlockNumber, Hash, Proof, H256, U256};

#[derive(Clone)]
pub struct StatusAgent(Arc<Mutex<CurrentStatus>>);

impl StatusAgent {
    pub fn new(status: CurrentStatus) -> Self {
        StatusAgent(Arc::new(Mutex::new(status)))
    }

    pub fn inner(&self) -> CurrentStatus {
        self.0.lock().clone()
    }

    pub fn swap(&self, new: CurrentStatus) {
        *self.0.lock() = new;
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct CurrentStatus {
    pub prev_hash:                  Hash,
    pub last_number:                BlockNumber,
    pub last_state_root:            H256,
    pub tx_num_limit:               u64,
    pub max_tx_size:                U256,
    pub proof:                      Proof,
    pub last_checkpoint_block_hash: Hash,
}
