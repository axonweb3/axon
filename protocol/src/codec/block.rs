use rlp::{Decodable, DecoderError, Encodable, Prototype, Rlp, RlpStream};

use crate::types::{
    Address, Block, Bloom, Bytes, Header, Proof, UnverifiedTransaction, Validator, H256, H64, U256,
};

impl Encodable for Header {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(16)
            .append(&self.parent_hash)
            .append(&self.uncles_hash)
            .append(&self.author)
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
            .append(&self.base_fee_per_gas);
    }
}

impl Decodable for Header {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(16) => {
                let parent_hash: H256 = r.val_at(0)?;
                let uncles_hash: H256 = r.val_at(1)?;
                let author: Address = r.val_at(2)?;
                let state_root: H256 = r.val_at(3)?;
                let transactions_root: H256 = r.val_at(4)?;
                let receipts_root: H256 = r.val_at(5)?;
                let log_bloom: Bloom = r.val_at(6)?;
                let difficulty: U256 = r.val_at(7)?;
                let timestamp: u64 = r.val_at(8)?;
                let number: u64 = r.val_at(9)?;
                let gas_used: U256 = r.val_at(10)?;
                let gas_limit: U256 = r.val_at(11)?;
                let extra_data: Bytes = r.val_at(12)?;
                let mixed_hash: Option<H256> = r.val_at(13)?;
                let nonce: H64 = r.val_at(14)?;
                let base_fee_per_gas: Option<U256> = r.val_at(15)?;

                Ok(Header {
                    parent_hash,
                    uncles_hash,
                    author,
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
                })
            }
            _ => Err(DecoderError::RlpInconsistentLengthAndData),
        }
    }
}

impl Encodable for Block {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3)
            .append(&self.header)
            .append_list(&self.transactions)
            .append_list(&self.uncles);
    }
}

impl Decodable for Block {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(3) => {
                let header: Header = r.val_at(0)?;
                let transactions: Vec<UnverifiedTransaction> = r.list_at(1)?;
                let uncles: Vec<Header> = r.list_at(2)?;

                Ok(Block {
                    header,
                    transactions,
                    uncles,
                })
            }
            _ => Err(DecoderError::RlpInconsistentLengthAndData),
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
            _ => Err(DecoderError::RlpInconsistentLengthAndData),
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
            _ => Err(DecoderError::RlpInconsistentLengthAndData),
        }
    }
}
