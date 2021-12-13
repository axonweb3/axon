use rlp::{Decodable, DecoderError, Encodable, Prototype, Rlp, RlpStream};

use crate::types::{Bloom, Hash, Log, MerkleRoot, Receipt, U256};

impl Encodable for Receipt {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(5)
            .append(&self.tx_hash)
            .append(&self.state_root)
            .append(&self.used_gas)
            .append(&self.logs_bloom)
            .append_list(&self.logs);
    }
}

impl Decodable for Receipt {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(5) => {
                let tx_hash: Hash = r.val_at(0)?;
                let state_root: MerkleRoot = r.val_at(1)?;
                let used_gas: U256 = r.val_at(2)?;
                let logs_bloom: Bloom = r.val_at(3)?;
                let logs: Vec<Log> = r.list_at(4)?;
                Ok(Receipt {
                    tx_hash,
                    state_root,
                    used_gas,
                    logs_bloom,
                    logs,
                })
            }
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
