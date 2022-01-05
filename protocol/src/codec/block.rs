use std::error::Error;

use overlord::Codec;
use rlp::{Decodable, DecoderError, Encodable, Prototype, Rlp, RlpStream};

use crate::types::{Block, Bytes, Header, Pill, Proof, Validator};
use crate::{codec::error::CodecError, ProtocolError};

impl Encodable for Header {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(19)
            .append(&self.prev_hash)
            .append(&self.proposer)
            .append(&self.state_root)
            .append(&self.transactions_root)
            .append(&self.signed_txs_hash)
            .append(&self.receipts_root)
            .append(&self.log_bloom)
            .append(&self.difficulty)
            .append(&self.timestamp)
            .append(&self.number)
            .append(&self.gas_used)
            .append(&self.gas_limit)
            .append(&self.extra_data)
            .append(&self.mixed_hash)
            .append(&self.nonce)
            .append(&self.base_fee_per_gas)
            .append(&self.proof)
            .append(&self.last_checkpoint_block_hash)
            .append(&self.chain_id);
    }
}

impl Decodable for Header {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(19) => Ok(Header {
                prev_hash:                  r.val_at(0)?,
                proposer:                   r.val_at(1)?,
                state_root:                 r.val_at(2)?,
                transactions_root:          r.val_at(3)?,
                signed_txs_hash:            r.val_at(4)?,
                receipts_root:              r.val_at(5)?,
                log_bloom:                  r.val_at(6)?,
                difficulty:                 r.val_at(7)?,
                timestamp:                  r.val_at(8)?,
                number:                     r.val_at(9)?,
                gas_used:                   r.val_at(10)?,
                gas_limit:                  r.val_at(11)?,
                extra_data:                 r.val_at(12)?,
                mixed_hash:                 r.val_at(13)?,
                nonce:                      r.val_at(14)?,
                base_fee_per_gas:           r.val_at(15)?,
                proof:                      r.val_at(16)?,
                last_checkpoint_block_hash: r.val_at(17)?,
                chain_id:                   r.val_at(18)?,
            }),
            _ => Err(DecoderError::RlpExpectedToBeList),
        }
    }
}

impl Encodable for Block {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2)
            .append(&self.header)
            .append_list(&self.tx_hashes);
    }
}

impl Decodable for Block {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(2) => Ok(Block {
                header:    r.val_at(0)?,
                tx_hashes: r.list_at(1)?,
            }),
            _ => Err(DecoderError::RlpExpectedToBeList),
        }
    }
}

impl Encodable for Proof {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(5)
            .append(&self.number)
            .append(&self.round)
            .append(&self.block_hash)
            .append(&self.signature)
            .append(&self.bitmap);
    }
}

impl Decodable for Proof {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(5) => Ok(Proof {
                number:     r.val_at(0)?,
                round:      r.val_at(1)?,
                block_hash: r.val_at(2)?,
                signature:  r.val_at(3)?,
                bitmap:     r.val_at(4)?,
            }),
            _ => Err(DecoderError::RlpExpectedToBeList),
        }
    }
}

impl Encodable for Validator {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3)
            .append(&self.pub_key)
            .append(&self.propose_weight)
            .append(&self.vote_weight);
    }
}

impl Decodable for Validator {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(3) => Ok(Validator {
                pub_key:        r.val_at(0)?,
                propose_weight: r.val_at(1)?,
                vote_weight:    r.val_at(2)?,
            }),
            _ => Err(DecoderError::RlpExpectedToBeList),
        }
    }
}

impl Encodable for Pill {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2)
            .append(&self.block)
            .append_list(&self.propose_hashes);
    }
}

impl Decodable for Pill {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(2) => Ok(Pill {
                block:          r.val_at(0)?,
                propose_hashes: r.list_at(1)?,
            }),
            _ => Err(DecoderError::RlpExpectedToBeList),
        }
    }
}

impl Codec for Pill {
    fn encode(&self) -> Result<Bytes, Box<dyn Error + Send>> {
        Ok(rlp::encode(self).freeze())
    }

    fn decode(data: Bytes) -> Result<Self, Box<dyn Error + Send>> {
        let ret: Pill = rlp::decode(data.as_ref())
            .map_err(|e| ProtocolError::from(CodecError::Rlp(e.to_string())))?;
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::MessageCodec;

    use super::*;

    #[test]
    fn test_block_codec() {
        let block = Block::default();
        let bytes = rlp::encode(&block);
        let decode: Block = rlp::decode(bytes.as_ref()).unwrap();
        assert_eq!(block, decode);
    }

    #[test]
    fn test_header_codec() {
        let header = Header::default();
        let bytes = rlp::encode(&header);
        let decode: Header = rlp::decode(bytes.as_ref()).unwrap();
        assert_eq!(header, decode);
    }

    #[test]
    fn test_proof_codec() {
        let mut proof = Proof::default();
        let bytes = proof.encode_msg().unwrap();
        let decode: Proof = Proof::decode_msg(bytes).unwrap();
        assert_eq!(proof, decode);
    }
}
