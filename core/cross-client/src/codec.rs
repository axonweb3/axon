use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

use crate::error::CrossChainError;
use crate::types::{Direction, FromCkbRequest, Transfer};

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

impl Encodable for FromCkbRequest {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_list(&self.0);
    }
}

impl Decodable for FromCkbRequest {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(FromCkbRequest(rlp.as_list()?))
    }
}
