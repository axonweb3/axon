use crate::types::{Bloom, Hash, MerkleRoot, U256};
pub use ethereum::Log;
pub use ethereum_types::BloomInput;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub tx_hash:      Hash,
    pub block_number: u64,
    pub block_hash:   Hash,
    pub tx_index:     u32,
    pub state_root:   MerkleRoot,
    pub used_gas:     U256,
    pub logs_bloom:   Bloom,
    pub logs:         Vec<Log>,
}
