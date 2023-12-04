pub use ethereum::Log;
pub use ethereum_types::BloomInput;

use crate::types::{Bloom, ExitReason, ExitSucceed, Hash, MerkleRoot, H160, U64};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub tx_hash:      Hash,
    pub block_number: u64,
    pub block_hash:   Hash,
    pub tx_index:     u32,
    pub state_root:   MerkleRoot,
    pub used_gas:     U64,
    pub logs_bloom:   Bloom,
    pub logs:         Vec<Log>,
    pub log_index:    u32,
    pub code_address: Option<Hash>,
    pub sender:       H160,
    pub ret:          ExitReason,
    pub removed:      bool,
}

impl Default for Receipt {
    fn default() -> Self {
        Receipt {
            tx_hash:      Default::default(),
            block_number: Default::default(),
            block_hash:   Default::default(),
            tx_index:     Default::default(),
            state_root:   Default::default(),
            used_gas:     Default::default(),
            logs_bloom:   Default::default(),
            logs:         Default::default(),
            log_index:    Default::default(),
            code_address: Default::default(),
            sender:       Default::default(),
            ret:          ExitReason::Succeed(ExitSucceed::Stopped),
            removed:      Default::default(),
        }
    }
}

impl Receipt {
    pub fn status(&self) -> U64 {
        match self.ret {
            ExitReason::Succeed(_) => U64::one(),
            _ => U64::zero(),
        }
    }
}
