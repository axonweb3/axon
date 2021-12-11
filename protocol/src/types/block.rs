use crate::types::{Address, Bloom, Bytes, Hash, MerkleRoot, H64, U256};

pub type BlockNumber = u64;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub header:    Header,
    pub tx_hashes: Vec<Hash>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub prev_hash:         Hash,
    pub proposer:          Address,
    pub state_root:        MerkleRoot,
    pub transactions_root: MerkleRoot,
    pub receipts_root:     MerkleRoot,
    pub log_bloom:         Bloom,
    pub difficulty:        U256,
    pub timestamp:         u64,
    pub number:            BlockNumber,
    pub gas_used:          U256,
    pub gas_limit:         U256,
    pub extra_data:        Bytes,
    pub mixed_hash:        Option<Hash>,
    pub nonce:             H64,
    pub base_fee_per_gas:  Option<U256>,
    pub proof:             Proof,
    pub chain_id:          u64,
}

#[derive(Default, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Proof {
    pub number:     u64,
    pub round:      u64,
    pub block_hash: Hash,
    pub signature:  Bytes,
    pub bitmap:     Bytes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pill {
    pub block:          Block,
    pub propose_hashes: Vec<Hash>,
}
