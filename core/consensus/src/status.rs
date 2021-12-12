use std::lazy::SyncOnceCell;
use std::sync::Arc;

use parking_lot::Mutex;

use common_merkle::Merkle;
use protocol::traits::Context;
use protocol::types::{
    BlockNumber, Bloom, ExecResponse, Hash, Hasher, MerkleRoot, Metadata, Proof, U256,
};
use protocol::Display;

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

#[derive(Clone, Debug, Display)]
#[display(
    fmt = "exec height {}, cycles used {}, state root {:?}, receipt root {:?}, confirm root {:?}",
    exec_height,
    gas_used,
    state_root,
    receipts_root,
    state_root
)]
pub struct ExecutedInfo {
    pub ctx:           Context,
    pub exec_height:   u64,
    pub gas_used:      u64,
    pub state_root:    MerkleRoot,
    pub receipts_root: MerkleRoot,
}

impl ExecutedInfo {
    pub fn new(ctx: Context, height: u64, state_root: MerkleRoot, resp: Vec<ExecResponse>) -> Self {
        let gas_sum = resp.iter().map(|r| r.remain_gas).sum();

        let receipt = Merkle::from_hashes(
            resp.iter()
                .map(|r| Hasher::digest(r.ret))
                .collect::<Vec<_>>(),
        )
        .get_root_hash()
        .unwrap_or_default();

        Self {
            ctx,
            exec_height: height,
            gas_used: gas_sum,
            receipts_root: receipt,
            state_root,
        }
    }
}
