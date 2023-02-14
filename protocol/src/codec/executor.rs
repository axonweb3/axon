use rlp::{Decodable, DecoderError, Encodable, Prototype, Rlp, RlpStream};

use crate::types::{TxResp, H160, H256, U256};

impl Encodable for TxResp {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(7)
            .append(&bincode::serialize(&self.exit_reason).unwrap())
            .append(&self.ret)
            .append(&self.gas_used)
            .append(&self.remain_gas)
            .append(&self.fee_cost)
            .append_list(&self.logs)
            .append(&self.code_address)
            .append(&self.removed);
    }
}

impl Decodable for TxResp {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(7) => Ok(TxResp {
                exit_reason:  {
                    let tmp: Vec<u8> = r.val_at(0)?;
                    bincode::deserialize(&tmp)
                        .map_err(|_| DecoderError::Custom("field exit reason"))?
                },
                ret:          r.val_at(1)?,
                gas_used:     r.val_at(2)?,
                remain_gas:   r.val_at(3)?,
                fee_cost:     r.val_at(4)?,
                logs:         r.list_at(5)?,
                code_address: r.val_at(6)?,
                removed:      r.val_at(7)?,
            }),
            _ => Err(DecoderError::RlpExpectedToBeList),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ExecutorContext;

    #[test]
    fn test_exec_ctx_codec() {
        let exec_ctx = ExecutorContext::default();
        let bytes = rlp::encode(&exec_ctx);
        assert_eq!(bytes, exec_ctx.rlp_bytes());
        let decode: ExecutorContext = rlp::decode(bytes.as_ref()).unwrap();
        assert_eq!(exec_ctx, decode);
    }
}
