use crate::types::{Bytes, Hash};
pub use ethereum::{BlockV2 as Block, Header, PartialHeader};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Proof {
    pub height:     u64,
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
