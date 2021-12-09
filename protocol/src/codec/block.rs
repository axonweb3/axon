use std::error::Error;

use overlord::Codec;
use rlp::{Decodable, DecoderError, Encodable, Prototype, Rlp, RlpStream};

use crate::types::{
    Address, Block, Bloom, Bytes, Hash, Header, Pill, Proof, UnverifiedTransaction, Validator,
    H256, H64, U256,
};
use crate::{codec::error::CodecError, ProtocolError};

impl Encodable for Header {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(16)
            .append(&self.prev_hash)
            .append(&self.proposer)
            .append(&self.state_root)
            .append(&self.transactions_root)
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
            .append(&self.proof);
    }
}

impl Decodable for Header {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(16) => {
                let prev_hash: H256 = r.val_at(0)?;
                let proposer: Address = r.val_at(1)?;
                let state_root: H256 = r.val_at(2)?;
                let transactions_root: H256 = r.val_at(3)?;
                let receipts_root: H256 = r.val_at(4)?;
                let log_bloom: Bloom = r.val_at(5)?;
                let difficulty: U256 = r.val_at(6)?;
                let timestamp: u64 = r.val_at(7)?;
                let number: u64 = r.val_at(8)?;
                let gas_used: U256 = r.val_at(9)?;
                let gas_limit: U256 = r.val_at(10)?;
                let extra_data: Bytes = r.val_at(11)?;
                let mixed_hash: Option<H256> = r.val_at(12)?;
                let nonce: H64 = r.val_at(13)?;
                let base_fee_per_gas: Option<U256> = r.val_at(14)?;
                let proof: Proof = r.val_at(15)?;

                Ok(Header {
                    prev_hash,
                    proposer,
                    state_root,
                    transactions_root,
                    receipts_root,
                    log_bloom,
                    difficulty,
                    timestamp,
                    number,
                    gas_used,
                    gas_limit,
                    extra_data,
                    mixed_hash,
                    nonce,
                    base_fee_per_gas,
                    proof,
                })
            }
            _ => Err(DecoderError::RlpExpectedToBeList),
        }
    }
}

impl Encodable for Block {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2)
            .append(&self.header)
            .append_list(&self.transactions);
    }
}

impl Decodable for Block {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(2) => {
                let header: Header = r.val_at(0)?;
                let transactions: Vec<UnverifiedTransaction> = r.list_at(1)?;

                Ok(Block {
                    header,
                    transactions,
                })
            }
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
            Prototype::List(5) => {
                let number: u64 = r.val_at(0)?;
                let round: u64 = r.val_at(1)?;
                let block_hash: H256 = r.val_at(2)?;
                let signature: Bytes = r.val_at(3)?;
                let bitmap: Bytes = r.val_at(4)?;

                Ok(Proof {
                    number,
                    round,
                    block_hash,
                    signature,
                    bitmap,
                })
            }
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
            Prototype::List(3) => {
                let pub_key: Bytes = r.val_at(0)?;
                let propose_weight: u32 = r.val_at(1)?;
                let vote_weight: u32 = r.val_at(2)?;

                Ok(Validator {
                    pub_key,
                    propose_weight,
                    vote_weight,
                })
            }
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
            Prototype::List(2) => {
                let block: Block = r.val_at(0)?;
                let propose_hashes: Vec<Hash> = r.list_at(1)?;

                Ok(Pill {
                    block,
                    propose_hashes,
                })
            }
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
}
