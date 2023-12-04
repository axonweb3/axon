use derive_more::Display;
use faster_hex::withpfx_lowercase;
use rlp_derive::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

#[cfg(feature = "hex-serialize")]
use crate::codec::serialize_uint;
use crate::types::{
    logs_bloom, Bloom, BloomInput, Bytes, ExecResp, Hash, Hasher, Log, MerkleRoot, Receipt,
    SignedTransaction, VecDisplayHelper, H160, U64,
};
use crate::{codec::ProtocolCodec, types::TypesError};

pub type BlockNumber = u64;

#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, PartialEq, Eq, Display)]
pub enum BlockVersion {
    #[default]
    V0,
}

impl From<BlockVersion> for u8 {
    fn from(value: BlockVersion) -> Self {
        match value {
            BlockVersion::V0 => 0,
        }
    }
}

impl TryFrom<u8> for BlockVersion {
    type Error = TypesError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BlockVersion::V0),
            _ => Err(TypesError::InvalidBlockVersion(value)),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq, Display)]
#[display(
    fmt = "Proposal {{ \
        version: {:?}, prev_hash: {:#x}, proposer: {:#x}, prev_state_root: {:#x}, \
        transactions_root: {:#x}, signed_txs_hash: {:#x}, timestamp: {}, number: {}, \
        gas_limit: {}, extra_data: {}, base_fee_per_gas: {}, proof: {}, \
        call_system_script_count: {}, chain_id: {} tx_hashes: {:?} \
    }}",
    version,
    prev_hash,
    proposer,
    prev_state_root,
    transactions_root,
    signed_txs_hash,
    timestamp,
    number,
    gas_limit,
    "VecDisplayHelper(&self.extra_data)",
    base_fee_per_gas,
    proof,
    call_system_script_count,
    chain_id,
    tx_hashes
)]
pub struct Proposal {
    pub version:                  BlockVersion,
    pub prev_hash:                Hash,
    pub proposer:                 H160,
    pub prev_state_root:          MerkleRoot,
    pub transactions_root:        MerkleRoot,
    pub signed_txs_hash:          Hash,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub timestamp:                u64,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub number:                   BlockNumber,
    pub gas_limit:                U64,
    pub extra_data:               Vec<ExtraData>,
    pub base_fee_per_gas:         U64,
    pub proof:                    Proof,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub chain_id:                 u64,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub call_system_script_count: u32,
    pub tx_hashes:                Vec<Hash>,
}

impl Proposal {
    pub fn hash(&self) -> Hash {
        Hasher::digest(self.encode().unwrap())
    }

    pub fn new_with_state_root(h: &Header, state_root: MerkleRoot, hashes: Vec<Hash>) -> Self {
        Proposal {
            version:                  h.version,
            prev_hash:                h.prev_hash,
            proposer:                 h.proposer,
            prev_state_root:          state_root,
            transactions_root:        h.transactions_root,
            signed_txs_hash:          h.signed_txs_hash,
            timestamp:                h.timestamp,
            number:                   h.number,
            gas_limit:                h.gas_limit,
            extra_data:               h.extra_data.clone(),
            base_fee_per_gas:         h.base_fee_per_gas,
            proof:                    h.proof.clone(),
            chain_id:                 h.chain_id,
            call_system_script_count: h.call_system_script_count,
            tx_hashes:                hashes,
        }
    }

    pub fn new_without_state_root(h: &Header) -> Self {
        Proposal {
            version:                  h.version,
            prev_hash:                h.prev_hash,
            proposer:                 h.proposer,
            prev_state_root:          Default::default(),
            transactions_root:        h.transactions_root,
            signed_txs_hash:          h.signed_txs_hash,
            timestamp:                h.timestamp,
            number:                   h.number,
            gas_limit:                h.gas_limit,
            extra_data:               h.extra_data.clone(),
            base_fee_per_gas:         h.base_fee_per_gas,
            proof:                    h.proof.clone(),
            chain_id:                 h.chain_id,
            call_system_script_count: h.call_system_script_count,
            tx_hashes:                vec![],
        }
    }
}

pub struct PackedTxHashes {
    pub hashes:                   Vec<Hash>,
    pub call_system_script_count: u32,
}

#[derive(
    RlpEncodable,
    RlpDecodable,
    Serialize,
    Deserialize,
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Display,
)]
#[display(fmt = "Block {{ header: {}, tx_hashes: {:?} }}", header, tx_hashes)]
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
            version:                  proposal.version,
            prev_hash:                proposal.prev_hash,
            proposer:                 proposal.proposer,
            state_root:               exec_resp.state_root,
            transactions_root:        proposal.transactions_root,
            signed_txs_hash:          proposal.signed_txs_hash,
            receipts_root:            exec_resp.receipt_root,
            log_bloom:                Bloom::from(BloomInput::Raw(
                rlp::encode_list(&logs).as_ref(),
            )),
            timestamp:                proposal.timestamp,
            number:                   proposal.number,
            gas_used:                 exec_resp.gas_used.into(),
            gas_limit:                proposal.gas_limit,
            extra_data:               proposal.extra_data,
            base_fee_per_gas:         proposal.base_fee_per_gas,
            proof:                    proposal.proof,
            call_system_script_count: proposal.call_system_script_count,
            chain_id:                 proposal.chain_id,
        };

        Block {
            header,
            tx_hashes: proposal.tx_hashes,
        }
    }

    pub fn hash(&self) -> Hash {
        self.header.hash()
    }

    pub fn generate_receipts_and_logs(
        &self,
        txs: &[SignedTransaction],
        resp: &ExecResp,
    ) -> (Vec<Receipt>, Vec<Vec<Log>>) {
        let mut log_index = 0;
        let receipts = txs
            .iter()
            .enumerate()
            .zip(resp.tx_resp.iter())
            .map(|((idx, tx), res)| {
                let receipt = Receipt {
                    tx_hash: tx.transaction.hash,
                    block_number: self.header.number,
                    block_hash: self.hash(),
                    tx_index: idx as u32,
                    state_root: self.header.state_root,
                    used_gas: U64::from(res.gas_used),
                    logs_bloom: logs_bloom(res.logs.iter()),
                    logs: res.logs.clone(),
                    log_index,
                    code_address: res.code_address,
                    sender: tx.sender,
                    ret: res.exit_reason.clone(),
                    removed: res.removed,
                };
                log_index += res.logs.len() as u32;
                receipt
            })
            .collect::<Vec<_>>();
        let logs = receipts.iter().map(|r| r.logs.clone()).collect::<Vec<_>>();
        (receipts, logs)
    }
}

#[derive(
    RlpEncodable,
    RlpDecodable,
    Serialize,
    Deserialize,
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Display,
)]
#[display(
    fmt = "Header {{ \
        version: {:?}, prev_hash: {:#x}, proposer: {:#x}, state_root: {:#x}, \
        transactions_root: {:#x}, signed_txs_hash: {:#x}, receipts_root: {:#x}, \
        log_bloom: {:#x}, timestamp: {}, number: {}, gas_used: {}, \
        gas_limit: {}, extra_data: {}, base_fee_per_gas: {}, proof: {}, \
        call_system_script_count: {}, chain_id: {} \
    }}",
    version,
    prev_hash,
    proposer,
    state_root,
    transactions_root,
    signed_txs_hash,
    receipts_root,
    log_bloom,
    timestamp,
    number,
    gas_used,
    gas_limit,
    "VecDisplayHelper(&self.extra_data)",
    base_fee_per_gas,
    proof,
    call_system_script_count,
    chain_id
)]
pub struct Header {
    pub version:                  BlockVersion,
    pub prev_hash:                Hash,
    pub proposer:                 H160,
    pub state_root:               MerkleRoot,
    pub transactions_root:        MerkleRoot,
    pub signed_txs_hash:          Hash,
    pub receipts_root:            MerkleRoot,
    pub log_bloom:                Bloom,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub timestamp:                u64,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub number:                   BlockNumber,
    pub gas_used:                 U64,
    pub gas_limit:                U64,
    /// Extra data for the block header
    /// The first index of extra_data is used to store hardfork information:
    /// `HardforkInfoInner`
    pub extra_data:               Vec<ExtraData>,
    pub base_fee_per_gas:         U64,
    pub proof:                    Proof,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub call_system_script_count: u32,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub chain_id:                 u64,
}

impl Header {
    pub fn size(&self) -> usize {
        self.encode().unwrap().len()
    }

    pub fn hash(&self) -> Hash {
        Hasher::digest(&self.encode().unwrap())
    }
}

#[derive(
    RlpEncodable,
    RlpDecodable,
    Serialize,
    Deserialize,
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Display,
)]
#[display(fmt = "0x{:x}", inner)]
pub struct ExtraData {
    #[cfg_attr(
        feature = "hex-serialize",
        serde(serialize_with = "withpfx_lowercase::serialize")
    )]
    pub inner: Bytes,
}

#[derive(
    RlpEncodable,
    RlpDecodable,
    Serialize,
    Deserialize,
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Display,
)]
#[display(
    fmt = "Proof {{ \
        number: {}, round: {}, block_hash: {:#x}, \
        signature: 0x{:x}, bitmap: 0x{:x} \
    }}",
    number,
    round,
    block_hash,
    signature,
    bitmap
)]
pub struct Proof {
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub number:     u64,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub round:      u64,
    pub block_hash: Hash,
    #[cfg_attr(
        feature = "hex-serialize",
        serde(serialize_with = "withpfx_lowercase::serialize")
    )]
    pub signature:  Bytes,
    #[cfg_attr(
        feature = "hex-serialize",
        serde(serialize_with = "withpfx_lowercase::serialize")
    )]
    pub bitmap:     Bytes,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RichBlock {
    pub block: Block,
    pub txs:   Vec<SignedTransaction>,
}

impl RichBlock {
    pub fn generate_receipts_and_logs(&self, resp: &ExecResp) -> (Vec<Receipt>, Vec<Vec<Log>>) {
        self.block.generate_receipts_and_logs(&self.txs, resp)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{
        primitive::default_max_contract_limit, Block, BlockVersion, ConsensusConfig, Header, Hex,
        Metadata, MetadataVersion, ProposeCount, RichBlock, ValidatorExtend, H160,
    };
    use std::{
        str::FromStr,
        time::{SystemTime, UNIX_EPOCH},
    };

    pub fn time_now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    #[test]
    fn test_invalid_version() {
        assert_eq!(BlockVersion::try_from(0).unwrap(), BlockVersion::V0);

        let ver = rand::random::<u8>();
        if ver != 0 {
            assert!(BlockVersion::try_from(ver).is_err());
        }
    }

    #[test]
    fn print_genesis() {
        let genesis = RichBlock {
            txs:   vec![],
            block: Block {
                tx_hashes: vec![],
                header:    Header {
                    version:                  Default::default(),
                    prev_hash:                Default::default(),
                    proposer:                 Default::default(),
                    state_root:               Default::default(),
                    transactions_root:        Default::default(),
                    signed_txs_hash:          Default::default(),
                    receipts_root:            Default::default(),
                    log_bloom:                Default::default(),
                    timestamp:                time_now(),
                    number:                   0,
                    gas_used:                 Default::default(),
                    gas_limit:                Default::default(),
                    extra_data:               Default::default(),
                    base_fee_per_gas:         Default::default(),
                    proof:                    Default::default(),
                    call_system_script_count: 0,
                    chain_id:                 0,
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
            verifier_list: vec![ValidatorExtend {
                bls_pub_key: Hex::from_str("0x04102947214862a503c73904deb5818298a186d68c7907bb609583192a7de6331493835e5b8281f4d9ee705537c0e765580e06f86ddce5867812fceb42eecefd209f0eddd0389d6b7b0100f00fb119ef9ab23826c6ea09aadcc76fa6cea6a32724").unwrap(),
                pub_key: Hex::from_str("0x02ef0cb0d7bc6c18b4bea1f5908d9106522b35ab3c399369605d4242525bda7e60").unwrap(),
                address: H160::default(),
                propose_weight: 1,
                vote_weight: 1,
            }],
            propose_counter: vec![ProposeCount {
                address: H160::default(),
                count: 0,
            }],
            consensus_config: ConsensusConfig {
                gas_limit: 4294967295,
                interval: 3000,
                propose_ratio: 15,
                prevote_ratio: 10,
                precommit_ratio: 10,
                brake_ratio: 10,
                tx_num_limit: 20000,
                max_tx_size: 1024,
                max_contract_limit: default_max_contract_limit()
            }
        };

        println!("{}", serde_json::to_string(&metadata).unwrap());
    }
}
