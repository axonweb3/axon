use rlp::{Decodable, DecoderError, Encodable, Prototype, Rlp, RlpStream};

use crate::types::{ExecutorContext, H256, U256};

impl Encodable for ExecutorContext {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(10)
            .append(&self.block_number)
            .append(&self.block_hash)
            .append(&self.block_coinbase)
            .append(&self.block_timestamp)
            .append(&self.chain_id)
            .append(&self.difficulty)
            .append(&self.origin)
            .append(&self.gas_price)
            .append(&self.block_gas_limit)
            .append(&self.block_base_fee_per_gas);
    }
}

impl Decodable for ExecutorContext {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(10) => {
                let block_number: U256 = r.val_at(0)?;
                let block_hash: H256 = r.val_at(1)?;
                let block_coinbase: U256 = r.val_at(2)?;
                let block_timestamp: U256 = r.val_at(3)?;
                let chain_id: U256 = r.val_at(4)?;
                let difficulty: U256 = r.val_at(5)?;
                let origin: H256 = r.val_at(6)?;
                let gas_price: U256 = r.val_at(7)?;
                let block_gas_limit: U256 = r.val_at(8)?;
                let block_base_fee_per_gas: U256 = r.val_at(9)?;

                Ok(ExecutorContext {
                    block_number,
                    block_hash,
                    block_coinbase,
                    block_timestamp,
                    chain_id,
                    difficulty,
                    origin,
                    gas_price,
                    block_gas_limit,
                    block_base_fee_per_gas,
                })
            }
            _ => return Err(DecoderError::RlpExpectedToBeList),
        }
    }
}
