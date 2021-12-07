use crate::types::{Address, Bloom, Bytes, Hash, UnverifiedTransaction, H256, H64, U256};

pub type BlockNumber = u64;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub header:       Header,
    pub transactions: Vec<UnverifiedTransaction>,
    pub uncles:       Vec<Header>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub parent_hash:       H256,
    pub uncles_hash:       H256,
    pub author:            Address,
    pub state_root:        H256,
    pub transactions_root: H256,
    pub receipts_root:     H256,
    pub log_bloom:         Bloom,
    pub difficulty:        U256,
    pub timestamp:         u64,
    pub number:            BlockNumber,
    pub gas_used:          U256,
    pub gas_limit:         U256,
    pub extra_data:        Bytes,
    pub mixed_hash:        Option<H256>,
    pub nonce:             H64,
    pub base_fee_per_gas:  Option<U256>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Proof {
    pub number:     u64,
    pub round:      u64,
    pub block_hash: Hash,
    pub signature:  Bytes,
    pub bitmap:     Bytes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Validator {
    pub pub_key:        Bytes,
    pub propose_weight: u32,
    pub vote_weight:    u32,
}
