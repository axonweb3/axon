use rlp::{Decodable, DecoderError, Encodable, Prototype, Rlp, RlpStream};

use crate::types::{ExecutorContext, ExitReason, Hash, Log, TxResp, H160, H256, U256};

impl Encodable for TxResp {
    fn rlp_append(&self, s: &mut RlpStream) {
        let reason = bincode::serialize(&self.remain_gas).unwrap();
        s.begin_list(6)
            .append(&reason)
            .append(&self.ret)
            .append(&self.gas_used)
            .append(&self.remain_gas)
            .append_list(&self.logs)
            .append(&self.code_address);
    }
}

impl Decodable for TxResp {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(6) => {
                let tmp: Vec<u8> = r.val_at(0)?;
                let exit_reason: ExitReason = bincode::deserialize(&tmp)
                    .map_err(|_| DecoderError::Custom("field exit reason"))?;
                let ret: Vec<u8> = r.val_at(1)?;
                let gas_used: u64 = r.val_at(2)?;
                let remain_gas: u64 = r.val_at(3)?;
                let logs: Vec<Log> = r.list_at(4)?;
                let code_address: Option<Hash> = r.val_at(5)?;

                Ok(TxResp {
                    exit_reason,
                    ret,
                    gas_used,
                    remain_gas,
                    logs,
                    code_address,
                })
            }
            _ => Err(DecoderError::RlpExpectedToBeList),
        }
    }
}

impl Encodable for ExecutorContext {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(11)
            .append(&self.block_number)
            .append(&self.block_hash)
            .append(&self.block_coinbase)
            .append(&self.block_timestamp)
            .append(&self.chain_id)
            .append(&self.difficulty)
            .append(&self.origin)
            .append(&self.gas_price)
            .append(&self.block_gas_limit)
            .append(&self.block_base_fee_per_gas)
            .append_list(&self.logs);
    }
}

impl Decodable for ExecutorContext {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(10) => {
                let block_number: U256 = r.val_at(0)?;
                let block_hash: H256 = r.val_at(1)?;
                let block_coinbase: H160 = r.val_at(2)?;
                let block_timestamp: U256 = r.val_at(3)?;
                let chain_id: U256 = r.val_at(4)?;
                let difficulty: U256 = r.val_at(5)?;
                let origin: H160 = r.val_at(6)?;
                let gas_price: U256 = r.val_at(7)?;
                let block_gas_limit: U256 = r.val_at(8)?;
                let block_base_fee_per_gas: U256 = r.val_at(9)?;
                let logs: Vec<Log> = r.list_at(10)?;

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
                    logs,
                })
            }
            _ => Err(DecoderError::RlpExpectedToBeList),
        }
    }
}
