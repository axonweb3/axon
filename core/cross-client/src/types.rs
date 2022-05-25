use serde::{Deserialize, Serialize};

use protocol::types::Hash;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Direction {
    FromCkb = 0u8,
    FromAxon = 1u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FromCkbRequest {
    pub ckb_tx_hash: Hash,
}

pub struct FromAxonRequest {}
