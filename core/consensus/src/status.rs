use std::lazy::SyncOnceCell;
use std::sync::Arc;

use parking_lot::Mutex;

use protocol::types::{BlockNumber, Bloom, Hash, MerkleRoot, Metadata, Proof, U256};

pub static METADATA_CONTROLER: SyncOnceCell<MetadataController> = SyncOnceCell::new();

#[derive(Default)]
pub struct MetadataController {
    current:  Arc<Mutex<Metadata>>,
    previous: Arc<Mutex<Metadata>>,
    next:     Arc<Mutex<Metadata>>,
}

impl MetadataController {
    pub fn init(
        current: Arc<Mutex<Metadata>>,
        previous: Arc<Mutex<Metadata>>,
        next: Arc<Mutex<Metadata>>,
    ) -> Self {
        MetadataController {
            current,
            previous,
            next,
        }
    }

    pub fn current(&self) -> Metadata {
        self.current.lock().clone()
    }

    pub fn previous(&self) -> Metadata {
        self.previous.lock().clone()
    }

    pub fn set_next(&self, next: Metadata) {
        *self.next.lock() = next;
    }

    pub fn update(&self, number: BlockNumber) {
        let current = self.current();

        if current.version.contains(number) {
            return;
        }

        let next = self.next.lock().clone();
        *self.previous.lock() = current;
        *self.current.lock() = next;
    }
}

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
    pub prev_hash:        Hash,
    pub last_number:      BlockNumber,
    pub state_root:       MerkleRoot,
    pub receipts_root:    MerkleRoot,
    pub log_bloom:        Bloom,
    pub gas_used:         U256,
    pub gas_limit:        U256,
    pub base_fee_per_gas: Option<U256>,
    pub proof:            Proof,
}
