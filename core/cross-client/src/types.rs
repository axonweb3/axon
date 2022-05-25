use serde::{Deserialize, Serialize};

use protocol::types::{Hash, H160};

use crate::error::CrossChainError;

#[repr(u8)]
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    FromCkb = 0u8,
    FromAxon = 1u8,
}

impl TryFrom<u8> for Direction {
    type Error = CrossChainError;

    fn try_from(d: u8) -> Result<Self, Self::Error> {
        match d {
            0 => Ok(Direction::FromCkb),
            1 => Ok(Direction::FromAxon),
            _ => Err(CrossChainError::InvalidDirection),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub direction:      Direction,
    pub tx_hash:        Hash,
    pub address:        H160,
    pub erc20_address:  H160,
    pub sudt_type_hash: Hash,
    pub ckb_amount:     u64,
    pub sudt_amount:    u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Requests(pub Vec<Transfer>);
