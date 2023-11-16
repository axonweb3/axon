pub use ethereum::{AccessList, AccessListItem, Account};
pub use evm::{backend::Log, Config, ExitError, ExitFatal, ExitReason, ExitRevert, ExitSucceed};
pub use hasher::HasherKeccak;

use rlp_derive::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::types::{
    Bloom, ExtraData, Hash, Hasher, Header, MerkleRoot, Proposal, H160, H256, U256,
};

use super::Hex;

const BLOOM_BYTE_LENGTH: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecResp {
    pub state_root:   MerkleRoot,
    pub receipt_root: MerkleRoot,
    pub gas_used:     u64,
    pub tx_resp:      Vec<TxResp>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxResp {
    pub exit_reason:  ExitReason,
    pub ret:          Vec<u8>,
    pub gas_used:     u64,
    pub remain_gas:   u64,
    pub fee_cost:     U256,
    pub logs:         Vec<Log>,
    pub code_address: Option<Hash>,
    pub removed:      bool,
}

impl Default for TxResp {
    fn default() -> Self {
        TxResp {
            exit_reason:  ExitReason::Succeed(ExitSucceed::Stopped),
            gas_used:     u64::default(),
            remain_gas:   u64::default(),
            fee_cost:     U256::default(),
            removed:      false,
            ret:          vec![],
            logs:         vec![],
            code_address: None,
        }
    }
}

#[derive(RlpEncodable, RlpDecodable, Default, Clone, Debug, PartialEq, Eq)]
pub struct ExecutorContext {
    pub block_number:           U256,
    pub block_coinbase:         H160,
    pub block_timestamp:        U256,
    pub chain_id:               U256,
    pub origin:                 H160,
    pub gas_price:              U256,
    pub block_gas_limit:        U256,
    pub block_base_fee_per_gas: U256,
    pub extra_data:             Vec<ExtraData>,
}

impl From<Proposal> for ExecutorContext {
    fn from(h: Proposal) -> Self {
        ExecutorContext {
            block_number:           h.number.into(),
            block_coinbase:         h.proposer,
            block_timestamp:        h.timestamp.into(),
            chain_id:               h.chain_id.into(),
            origin:                 h.proposer,
            gas_price:              U256::one(),
            block_gas_limit:        h.gas_limit,
            block_base_fee_per_gas: h.base_fee_per_gas,
            extra_data:             h.extra_data,
        }
    }
}

impl From<&Header> for ExecutorContext {
    fn from(h: &Header) -> ExecutorContext {
        ExecutorContext {
            block_number:           h.number.into(),
            block_coinbase:         h.proposer,
            block_timestamp:        h.timestamp.into(),
            chain_id:               h.chain_id.into(),
            origin:                 h.proposer,
            gas_price:              U256::one(),
            block_gas_limit:        h.gas_limit,
            block_base_fee_per_gas: h.base_fee_per_gas,
            extra_data:             h.extra_data.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EthAccountProof {
    pub balance:       U256,
    pub code_hash:     H256,
    pub nonce:         U256,
    pub storage_hash:  H256,
    pub account_proof: Vec<Hex>,
    pub storage_proof: Vec<EthStorageProof>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EthStorageProof {
    pub key:   H256,
    pub value: H256,
    pub proof: Vec<Hex>,
}

pub fn logs_bloom<'a, I>(logs: I) -> Bloom
where
    I: Iterator<Item = &'a Log>,
{
    let mut bloom = Bloom::zero();

    for log in logs {
        m3_2048(&mut bloom, log.address.as_bytes());
        for topic in log.topics.iter() {
            m3_2048(&mut bloom, topic.as_bytes());
        }
    }
    bloom
}

fn m3_2048(bloom: &mut Bloom, x: &[u8]) {
    let hash = Hasher::digest(x).0;
    for i in [0, 2, 4] {
        let bit = (hash[i + 1] as usize + ((hash[i] as usize) << 8)) & 0x7FF;
        bloom.0[BLOOM_BYTE_LENGTH - 1 - bit / 8] |= 1 << (bit % 8);
    }
}
