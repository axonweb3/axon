use parking_lot::Mutex;
use protocol::types::H256;
use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    sync::Arc,
};

use super::web3_types::{Filter, Web3Log};

pub type BlockNumber = u64;
const MAX_BLOCK_HISTORY_SIZE: usize = 32;
/// Thread-safe filter state.
#[derive(Clone)]
pub struct SyncPollFilter(Arc<Mutex<PollFilter>>);

impl SyncPollFilter {
    /// New `SyncPollFilter`
    pub fn new(f: PollFilter) -> Self {
        SyncPollFilter(Arc::new(Mutex::new(f)))
    }

    /// Modify underlying filter
    pub fn modify<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut PollFilter) -> R,
    {
        f(&mut self.0.lock())
    }
}

/// Filter state.
#[derive(Clone)]
pub enum PollFilter {
    /// Number of last block which client was notified about.
    Block {
        last_block_number:      BlockNumber,
        #[doc(hidden)]
        recent_reported_hashes: VecDeque<(BlockNumber, H256)>,
    },
    /// Hashes of all pending transactions the client knows about.
    PendingTransaction(BTreeSet<H256>),
    /// Number of From block number, last seen block hash, pending logs and log
    /// filter itself.
    Logs {
        block_number:    BlockNumber,
        last_block_hash: Option<H256>,
        previous_logs:   HashSet<Web3Log>,
        filter:          Filter,
        include_pending: bool,
    },
}

impl PollFilter {
    pub fn max_block_history_size() -> usize {
        MAX_BLOCK_HISTORY_SIZE
    }
}
