use std::error::Error;

use overlord::Codec;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

use crate::types::{BlockVersion, Bytes, Proposal, BASE_FEE_PER_GAS, MAX_BLOCK_GAS_LIMIT};
use crate::{codec::error::CodecError, lazy::CHAIN_ID, ProtocolError};

impl Encodable for BlockVersion {
    fn rlp_append(&self, s: &mut RlpStream) {
        let ver: u8 = (*self).into();
        s.begin_list(1).append(&ver);
    }
}

impl Decodable for BlockVersion {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        let ver: u8 = r.val_at(0)?;
        ver.try_into()
            .map_err(|_| DecoderError::Custom("Invalid block version"))
    }
}

impl Encodable for Proposal {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(12)
            .append(&self.version)
            .append(&self.prev_hash)
            .append(&self.proposer)
            .append(&self.prev_state_root)
            .append(&self.transactions_root)
            .append(&self.signed_txs_hash)
            .append(&self.timestamp)
            .append(&self.number)
            .append_list(&self.extra_data)
            .append(&self.proof)
            .append(&self.call_system_script_count)
            .append_list(&self.tx_hashes);
    }
}

impl Decodable for Proposal {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        Ok(Proposal {
            version:                  r.val_at(0)?,
            prev_hash:                r.val_at(1)?,
            proposer:                 r.val_at(2)?,
            prev_state_root:          r.val_at(3)?,
            transactions_root:        r.val_at(4)?,
            signed_txs_hash:          r.val_at(5)?,
            timestamp:                r.val_at(6)?,
            number:                   r.val_at(7)?,
            gas_limit:                MAX_BLOCK_GAS_LIMIT.into(),
            extra_data:               r.list_at(8)?,
            base_fee_per_gas:         BASE_FEE_PER_GAS.into(),
            proof:                    r.val_at(9)?,
            chain_id:                 **CHAIN_ID.load(),
            call_system_script_count: r.val_at(10)?,
            tx_hashes:                r.list_at(11)?,
        })
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
    use crate::types::{Block, Bytes, ExtraData, Header, Proof, H160, H256};
    use rand::random;

    use super::*;

    fn rand_bytes(len: usize) -> Bytes {
        (0..len).map(|_| random::<u8>()).collect::<Vec<_>>().into()
    }

    #[test]
    fn test_version_codec() {
        let ver = BlockVersion::V0;
        let bytes = rlp::encode(&ver);
        assert_eq!(bytes, ver.rlp_bytes());
        let decode: BlockVersion = rlp::decode(bytes.as_ref()).unwrap();
        assert_eq!(ver, decode);
    }

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
        let mock_proof = Proof {
            number:     random(),
            round:      random(),
            block_hash: H256::random(),
            signature:  rand_bytes(65),
            bitmap:     rand_bytes(32),
        };
        let mut proposal = Proposal {
            version:                  BlockVersion::V0,
            prev_hash:                H256::random(),
            proposer:                 H160::random(),
            prev_state_root:          H256::random(),
            transactions_root:        H256::random(),
            signed_txs_hash:          H256::random(),
            timestamp:                random(),
            number:                   random(),
            gas_limit:                30000000u64.into(),
            extra_data:               vec![ExtraData {
                inner: H256::random().0.to_vec().into(),
            }],
            base_fee_per_gas:         1337u64.into(),
            proof:                    mock_proof,
            chain_id:                 0,
            call_system_script_count: random(),
            tx_hashes:                vec![H256::random()],
        };
        let bytes = proposal.encode_msg().unwrap();
        let decode: Proposal = Proposal::decode_msg(bytes).unwrap();
        assert_eq!(proposal, decode);
    }
}
