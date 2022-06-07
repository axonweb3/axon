use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
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

impl Direction {
    pub fn is_from_ckb(&self) -> bool {
        self == &Direction::FromCkb
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

impl Encodable for Transfer {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(7)
            .append(&(self.direction as u8))
            .append(&self.tx_hash)
            .append(&self.address)
            .append(&self.erc20_address)
            .append(&self.sudt_type_hash)
            .append(&self.ckb_amount)
            .append(&self.sudt_amount);
    }
}

impl Decodable for Transfer {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(Transfer {
            direction:      rlp
                .val_at::<u8>(0)?
                .try_into()
                .map_err(|_| DecoderError::Custom("Invalid transfer direction"))?,
            tx_hash:        rlp.val_at(1)?,
            address:        rlp.val_at(2)?,
            erc20_address:  rlp.val_at(3)?,
            sudt_type_hash: rlp.val_at(4)?,
            ckb_amount:     rlp.val_at(5)?,
            sudt_amount:    rlp.val_at(6)?,
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

#[cfg(test)]
mod tests {
    use protocol::types::{Hash, H160};
    use rand::random;

    use super::*;

    fn random_transfer() -> Transfer {
        Transfer {
            direction:      0u8.try_into().unwrap(),
            tx_hash:        Hash::random(),
            address:        H160::random(),
            erc20_address:  H160::random(),
            sudt_type_hash: Hash::random(),
            ckb_amount:     random(),
            sudt_amount:    random(),
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
