use crate::types::BlockVersion;
#[cfg(feature = "proof")]
use crate::types::{Proposal, Vote};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

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

#[cfg(feature = "proof")]
impl Encodable for Vote {
    fn rlp_append(&self, s: &mut RlpStream) {
        let vote_type: u8 = self.vote_type;
        s.begin_list(4)
            .append(&self.height)
            .append(&self.round)
            .append(&vote_type)
            .append(&self.block_hash.to_vec());
    }
}

#[cfg(feature = "proof")]
impl Encodable for Proposal {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(13)
            .append(&self.version)
            .append(&self.prev_hash)
            .append(&self.proposer)
            .append(&self.prev_state_root)
            .append(&self.transactions_root)
            .append(&self.signed_txs_hash)
            .append(&self.timestamp)
            .append(&self.number)
            .append(&self.gas_limit.as_u64())
            .append_list(&self.extra_data)
            .append(&self.proof)
            .append(&self.call_system_script_count)
            .append_list(&self.tx_hashes);
    }
}
