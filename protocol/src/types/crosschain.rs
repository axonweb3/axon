use serde::{Deserialize, Serialize};

use crate::types::{TypesError, H160, H256};

#[repr(u8)]
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    FromCkb = 0u8,
    FromAxon = 1u8,
}

impl TryFrom<u8> for Direction {
    type Error = TypesError;

    fn try_from(d: u8) -> Result<Self, Self::Error> {
        match d {
            0 => Ok(Direction::FromCkb),
            1 => Ok(Direction::FromAxon),
            _ => Err(TypesError::InvalidDirection),
        }
    }
}

impl Direction {
    pub fn is_from_ckb(&self) -> bool {
        self == &Direction::FromCkb
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub direction:     Direction,
    pub address:       H160,
    pub erc20_address: H160,
    pub sudt_amount:   u128,
    pub ckb_amount:    u64,
    pub tx_hash:       H256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Requests(pub Vec<Transfer>);

impl Requests {
    pub fn direction(&self) -> Direction {
        self.0[0].direction
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RequestTxHashes {
    pub direction: Direction,
    pub tx_hashes: Vec<H256>,
}
