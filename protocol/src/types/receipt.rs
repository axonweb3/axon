use crate::types::{Bloom, Hash, MerkleRoot, H256, U256};
pub use ethereum::Log;
pub use ethereum_types::BloomInput;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub tx_hash:    Hash,
    pub state_root: MerkleRoot,
    pub used_gas:   U256,
    pub logs_bloom: Bloom,
    pub logs:       Vec<Log>,
}
