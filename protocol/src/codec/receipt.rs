use parity_scale_codec::{Decode, Encode};
use rlp::{Decodable, DecoderError, Encodable, Prototype, Rlp, RlpStream};

use crate::types::{ExitReason, Receipt};

impl Encodable for Receipt {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(11)
            .append(&self.tx_hash)
            .append(&self.block_number)
            .append(&self.block_hash)
            .append(&self.tx_index)
            .append(&self.state_root)
            .append(&self.used_gas)
            .append(&self.logs_bloom)
            .append_list(&self.logs)
            .append(&self.code_address)
            .append(&self.sender)
            .append(&self.ret.encode());
    }
}

impl Decodable for Receipt {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(11) => Ok(Receipt {
                tx_hash:      r.val_at(0)?,
                block_number: r.val_at(1)?,
                block_hash:   r.val_at(2)?,
                tx_index:     r.val_at(3)?,
                state_root:   r.val_at(4)?,
                used_gas:     r.val_at(5)?,
                logs_bloom:   r.val_at(6)?,
                logs:         r.list_at(7)?,
                code_address: r.val_at(8)?,
                sender:       r.val_at(9)?,
                ret:          {
                    let raw: Vec<u8> = r.val_at(10)?;
                    ExitReason::decode(&mut raw.as_slice())
                        .map_err(|_| DecoderError::Custom("Decode exit reason"))?
                },
            }),
            _ => Err(DecoderError::RlpExpectedToBeList),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_codec() {
        let block = Receipt::default();
        let bytes = rlp::encode(&block);
        let decode: Receipt = rlp::decode(bytes.as_ref()).unwrap();
        assert_eq!(block, decode);
    }
}
