pub use ethereum::Log;
pub use ethereum_types::BloomInput;

use crate::types::{Bloom, ExitReason, ExitSucceed, Hash, MerkleRoot, H160, U256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub tx_hash:      Hash,
    pub block_number: u64,
    pub block_hash:   Hash,
    pub tx_index:     u32,
    pub state_root:   MerkleRoot,
    pub used_gas:     U256,
    pub logs_bloom:   Bloom,
    pub logs:         Vec<Log>,
    pub code_address: Option<Hash>,
    pub sender:       H160,
    pub ret:          ExitReason,
}

impl Default for Receipt {
    fn default() -> Self {
        Receipt {
            ret: ExitReason::Succeed(ExitSucceed::Stopped),
            tx_hash: Default::default(),
            ..Default::default()
        }
    }
}

impl Receipt {
    pub fn status(&self) -> U256 {
        match self.ret {
            ExitReason::Succeed(_) => U256::one(),
            _ => U256::zero(),
        }
    }
}
