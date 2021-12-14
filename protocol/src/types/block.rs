use serde::{Deserialize, Serialize};

use crate::types::{Bloom, Bytes, Hash, MerkleRoot, SignedTransaction, H160, H64, U256};

pub type BlockNumber = u64;

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub header:    Header,
    pub tx_hashes: Vec<Hash>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub prev_hash:         Hash,
    pub proposer:          H160,
    pub state_root:        MerkleRoot,
    pub transactions_root: MerkleRoot,
    pub signed_txs_hash:   Hash,
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

#[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Genesis {
    pub block:    Block,
    pub rich_txs: Vec<SignedTransaction>,
}
