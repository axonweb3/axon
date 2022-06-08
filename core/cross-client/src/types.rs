use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use serde::{Deserialize, Serialize};

use protocol::types::{H160, H256};

use crate::{crosschain_abi, error::CrossChainError};

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

impl Direction {
    pub fn is_from_ckb(&self) -> bool {
        self == &Direction::FromCkb
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub direction: Direction,

    pub address:       H160,
    pub erc20_address: H160,
    pub sudt_amount:   u128,
    pub ckb_amount:    u64,
    pub tx_hash:       H256,
}

impl Encodable for Transfer {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(6)
            .append(&(self.direction as u8))
            .append(&self.tx_hash)
            .append(&self.address)
            .append(&self.erc20_address)
            .append(&self.ckb_amount)
            .append(&self.sudt_amount);
    }
}

impl Decodable for Transfer {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(Transfer {
            direction:     rlp
                .val_at::<u8>(0)?
                .try_into()
                .map_err(|_| DecoderError::Custom("Invalid transfer direction"))?,
            tx_hash:       rlp.val_at(1)?,
            address:       rlp.val_at(2)?,
            erc20_address: rlp.val_at(3)?,
            ckb_amount:    rlp.val_at(4)?,
            sudt_amount:   rlp.val_at(5)?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Requests(pub Vec<Transfer>);

impl Encodable for Requests {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_list(&self.0);
    }
}

impl Decodable for Requests {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(Requests(rlp.as_list()?))
    }
}

impl Requests {
    pub fn direction(&self) -> Direction {
        self.0[0].direction
    }
}

impl From<crosschain_abi::CrossFromCKBFilter> for Requests {
    fn from(logs: crosschain_abi::CrossFromCKBFilter) -> Self {
        Requests(
            logs.records
                .into_iter()
                .map(|r| Transfer {
                    direction:     Direction::FromCkb,
                    address:       r.0,
                    erc20_address: r.1,
                    sudt_amount:   r.2.as_u128(),
                    ckb_amount:    r.3.as_u64(),
                    tx_hash:       H256(r.4),
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use protocol::types::{Hash, H160};
    use rand::random;

    use super::*;

    fn random_transfer() -> Transfer {
        Transfer {
            direction:     0u8.try_into().unwrap(),
            tx_hash:       Hash::random(),
            address:       H160::random(),
            erc20_address: H160::random(),
            ckb_amount:    random(),
            sudt_amount:   random(),
        }
    }

    #[test]
    fn test_transfer_codec() {
        let origin = random_transfer();
        let raw = rlp::encode(&origin);
        let decode = rlp::decode::<Transfer>(&raw.freeze()).unwrap();
        assert_eq!(origin, decode);
    }

    #[test]
    fn test_requests_codec() {
        let origin = Requests((0..10).map(|_| random_transfer()).collect());
        let raw = rlp::encode(&origin);
        let decode = rlp::decode::<Requests>(&raw.freeze()).unwrap();
        assert_eq!(origin, decode);
    }
}
