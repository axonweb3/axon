use rlp_derive::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::codec::ProtocolCodec;
use crate::types::{
    Bloom, BloomInput, Bytes, ExecResp, Hash, Hasher, MerkleRoot, SignedTransaction, H160, H64,
    U256,
};

pub type BlockNumber = u64;

pub const MAX_BLOCK_GAS_LIMIT: u64 = 30_000_000;
pub const MAX_RPC_GAS_CAP: u64 = 50_000_000;
pub const BASE_FEE_PER_GAS: u64 = 0x539;

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub prev_hash:                  Hash,
    pub proposer:                   H160,
    pub transactions_root:          MerkleRoot,
    pub signed_txs_hash:            Hash,
    pub timestamp:                  u64,
    pub number:                     BlockNumber,
    pub gas_limit:                  U256,
    pub extra_data:                 Bytes,
    pub mixed_hash:                 Option<Hash>,
    pub base_fee_per_gas:           U256,
    pub proof:                      Proof,
    pub last_checkpoint_block_hash: Hash,
    pub chain_id:                   u64,
    pub call_system_script_count:   u32,
    pub tx_hashes:                  Vec<Hash>,
}

impl From<&Block> for Proposal {
    fn from(b: &Block) -> Self {
        Proposal {
            prev_hash:                  b.header.prev_hash,
            proposer:                   b.header.proposer,
            transactions_root:          b.header.transactions_root,
            signed_txs_hash:            b.header.signed_txs_hash,
            timestamp:                  b.header.timestamp,
            number:                     b.header.number,
            gas_limit:                  b.header.gas_limit,
            extra_data:                 b.header.extra_data.clone(),
            mixed_hash:                 b.header.mixed_hash,
            base_fee_per_gas:           b.header.base_fee_per_gas,
            proof:                      b.header.proof.clone(),
            last_checkpoint_block_hash: b.header.last_checkpoint_block_hash,
            chain_id:                   b.header.chain_id,
            call_system_script_count:   b.header.call_system_script_count,
            tx_hashes:                  b.tx_hashes.clone(),
        }
    }
}

impl From<Header> for Proposal {
    fn from(h: Header) -> Self {
        Proposal {
            prev_hash:                  h.prev_hash,
            proposer:                   h.proposer,
            transactions_root:          h.transactions_root,
            signed_txs_hash:            h.signed_txs_hash,
            timestamp:                  h.timestamp,
            number:                     h.number,
            gas_limit:                  h.gas_limit,
            extra_data:                 h.extra_data,
            mixed_hash:                 h.mixed_hash,
            base_fee_per_gas:           h.base_fee_per_gas,
            proof:                      h.proof,
            last_checkpoint_block_hash: h.last_checkpoint_block_hash,
            chain_id:                   h.chain_id,
            call_system_script_count:   h.call_system_script_count,
            tx_hashes:                  vec![],
        }
    }
}

impl Proposal {
    pub fn hash(&self) -> Hash {
        Hasher::digest(self.encode().unwrap())
    }
}

pub struct PackedTxHashes {
    pub hashes:                   Vec<Hash>,
    pub call_system_script_count: u32,
}

#[derive(
    RlpEncodable, RlpDecodable, Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq,
)]
pub struct Block {
    pub header:    Header,
    pub tx_hashes: Vec<Hash>,
}

impl Block {
    pub fn new(proposal: Proposal, exec_resp: ExecResp) -> Self {
        let logs = exec_resp
            .tx_resp
            .iter()
            .map(|r| Bloom::from(BloomInput::Raw(rlp::encode_list(&r.logs).as_ref())))
            .collect::<Vec<_>>();
        let header = Header {
            prev_hash:                  proposal.prev_hash,
            proposer:                   proposal.proposer,
            state_root:                 exec_resp.state_root,
            transactions_root:          proposal.transactions_root,
            signed_txs_hash:            proposal.signed_txs_hash,
            receipts_root:              exec_resp.receipt_root,
            log_bloom:                  Bloom::from(BloomInput::Raw(
                rlp::encode_list(&logs).as_ref(),
            )),
            difficulty:                 U256::one(),
            timestamp:                  proposal.timestamp,
            number:                     proposal.number,
            gas_used:                   exec_resp.gas_used.into(),
            gas_limit:                  proposal.gas_limit,
            extra_data:                 proposal.extra_data,
            mixed_hash:                 proposal.mixed_hash,
            nonce:                      Default::default(),
            base_fee_per_gas:           proposal.base_fee_per_gas,
            proof:                      proposal.proof,
            last_checkpoint_block_hash: proposal.last_checkpoint_block_hash,
            call_system_script_count:   proposal.call_system_script_count,
            chain_id:                   proposal.chain_id,
        };

        Block {
            header,
            tx_hashes: proposal.tx_hashes,
        }
    }

    pub fn hash(&self) -> Hash {
        Proposal::from(self).hash()
    }
}

#[derive(
    RlpEncodable, RlpDecodable, Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq,
)]
pub struct Header {
    pub prev_hash:                  Hash,
    pub proposer:                   H160,
    pub state_root:                 MerkleRoot,
    pub transactions_root:          MerkleRoot,
    pub signed_txs_hash:            Hash,
    pub receipts_root:              MerkleRoot,
    pub log_bloom:                  Bloom,
    pub difficulty:                 U256,
    pub timestamp:                  u64,
    pub number:                     BlockNumber,
    pub gas_used:                   U256,
    pub gas_limit:                  U256,
    pub extra_data:                 Bytes,
    pub mixed_hash:                 Option<Hash>,
    pub nonce:                      H64,
    pub base_fee_per_gas:           U256,
    pub proof:                      Proof,
    pub last_checkpoint_block_hash: Hash,
    pub call_system_script_count:   u32,
    pub chain_id:                   u64,
}

impl Header {
    pub fn size(&self) -> usize {
        self.encode().unwrap().len()
    }
}

#[derive(
    RlpEncodable, RlpDecodable, Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq,
)]
pub struct Proof {
    pub number:     u64,
    pub round:      u64,
    pub block_hash: Hash,
    pub signature:  Bytes,
    pub bitmap:     Bytes,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RichBlock {
    pub block: Block,
    pub txs:   Vec<SignedTransaction>,
}

#[cfg(test)]
mod tests {
    use crate::types::{
        Block, Header, Hex, Metadata, MetadataVersion, RichBlock, ValidatorExtend, H160,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn time_now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    #[test]
    fn print_genesis() {
        let genesis = RichBlock {
            txs:   vec![],
            block: Block {
                tx_hashes: vec![],
                header:    Header {
                    prev_hash:                  Default::default(),
                    proposer:                   Default::default(),
                    state_root:                 Default::default(),
                    transactions_root:          Default::default(),
                    signed_txs_hash:            Default::default(),
                    receipts_root:              Default::default(),
                    log_bloom:                  Default::default(),
                    difficulty:                 Default::default(),
                    timestamp:                  time_now(),
                    number:                     0,
                    gas_used:                   Default::default(),
                    gas_limit:                  Default::default(),
                    extra_data:                 Default::default(),
                    mixed_hash:                 Default::default(),
                    nonce:                      Default::default(),
                    base_fee_per_gas:           Default::default(),
                    proof:                      Default::default(),
                    last_checkpoint_block_hash: Default::default(),
                    call_system_script_count:   0,
                    chain_id:                   0,
                },
            },
        };

        println!("{}", serde_json::to_string(&genesis).unwrap());
    }

    #[test]
    fn print_metadata() {
        let metadata = Metadata {
            version: MetadataVersion::new(0, 1000000000),
            epoch: 0,
            gas_limit: 4294967295,
            gas_price: 1,
            interval: 3000,
            propose_ratio: 15,
            prevote_ratio: 10,
            precommit_ratio: 10,
            brake_ratio: 10,
            tx_num_limit: 20000,
            max_tx_size: 1024,
            verifier_list: vec![ValidatorExtend {
                bls_pub_key: Hex::from_string("0x04102947214862a503c73904deb5818298a186d68c7907bb609583192a7de6331493835e5b8281f4d9ee705537c0e765580e06f86ddce5867812fceb42eecefd209f0eddd0389d6b7b0100f00fb119ef9ab23826c6ea09aadcc76fa6cea6a32724".to_string()).unwrap(),
                pub_key: Hex::from_string("0x02ef0cb0d7bc6c18b4bea1f5908d9106522b35ab3c399369605d4242525bda7e60".to_string()).unwrap(),
                address: H160::default(),
                propose_weight: 1,
                vote_weight: 1,
            }],
            last_checkpoint_block_hash : Default::default(),
        };

        println!("{}", serde_json::to_string(&metadata).unwrap());
    }
}
