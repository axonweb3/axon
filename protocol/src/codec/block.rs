use std::error::Error;

use overlord::Codec;
use rlp::{Decodable, DecoderError, Encodable, Prototype, Rlp, RlpStream};

use crate::types::{Bytes, Proposal, BASE_FEE_PER_GAS, MAX_BLOCK_GAS_LIMIT};
use crate::{codec::error::CodecError, lazy::CHAIN_ID, ProtocolError};

impl Encodable for Proposal {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(10)
            .append(&self.prev_hash)
            .append(&self.proposer)
            .append(&self.transactions_root)
            .append(&self.signed_txs_hash)
            .append(&self.timestamp)
            .append(&self.number)
            .append(&self.proof)
            .append(&self.last_checkpoint_block_hash)
            .append(&self.call_system_script_count)
            .append_list(&self.tx_hashes);
    }
}

impl Decodable for Proposal {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(10) => Ok(Proposal {
                prev_hash:                  r.val_at(0)?,
                proposer:                   r.val_at(1)?,
                transactions_root:          r.val_at(2)?,
                signed_txs_hash:            r.val_at(3)?,
                timestamp:                  r.val_at(4)?,
                number:                     r.val_at(5)?,
                gas_limit:                  MAX_BLOCK_GAS_LIMIT.into(),
                extra_data:                 Default::default(),
                mixed_hash:                 None,
                base_fee_per_gas:           BASE_FEE_PER_GAS.into(),
                proof:                      r.val_at(6)?,
                last_checkpoint_block_hash: r.val_at(7)?,
                chain_id:                   **CHAIN_ID.load(),
                call_system_script_count:   r.val_at(8)?,
                tx_hashes:                  r.list_at(9)?,
            }),
            _ => Err(DecoderError::RlpInconsistentLengthAndData),
        }
    }
}

impl Codec for Proposal {
    fn encode(&self) -> Result<Bytes, Box<dyn Error + Send>> {
        Ok(rlp::encode(self).freeze())
    }

    fn decode(data: Bytes) -> Result<Self, Box<dyn Error + Send>> {
        let proposal: Proposal = rlp::decode(data.as_ref())
            .map_err(|e| ProtocolError::from(CodecError::Rlp(e.to_string())))?;
        Ok(proposal)
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::MessageCodec;
    use crate::types::{Block, Header, Proof};

    use super::*;

    #[test]
    fn test_block_codec() {
        let block = Block::default();
        let bytes = rlp::encode(&block);
        assert_eq!(bytes, block.rlp_bytes());
        let decode: Block = rlp::decode(bytes.as_ref()).unwrap();
        assert_eq!(block, decode);
    }

    #[test]
    fn test_header_codec() {
        let header = Header::default();
        let bytes = rlp::encode(&header);
        assert_eq!(bytes, header.rlp_bytes());
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

    #[test]
    fn test_proposal_codec() {
        let mut proposal = Proposal {
            gas_limit: 30000000u64.into(),
            base_fee_per_gas: 1337u64.into(),
            ..Default::default()
        };
        let bytes = proposal.encode_msg().unwrap();
        let decode: Proposal = Proposal::decode_msg(bytes).unwrap();
        assert_eq!(proposal, decode);
    }
}
