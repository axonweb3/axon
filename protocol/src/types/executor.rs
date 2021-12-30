pub use ethereum::{AccessList, AccessListItem, Account};
pub use evm::{backend::Log, Config, ExitError, ExitFatal, ExitReason, ExitRevert, ExitSucceed};

use crate::codec::ProtocolCodec;
use crate::types::{Hash, Hasher, Header, MerkleRoot, H160, U256};

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
    pub logs:         Vec<Log>,
    pub code_address: Option<Hash>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ExecutorContext {
    pub block_number:           U256,
    pub block_hash:             Hash,
    pub block_coinbase:         H160,
    pub block_timestamp:        U256,
    pub chain_id:               U256,
    pub difficulty:             U256,
    pub origin:                 H160,
    pub gas_price:              U256,
    pub block_gas_limit:        U256,
    pub block_base_fee_per_gas: U256,
    pub logs:                   Vec<Log>,
}

impl From<Header> for ExecutorContext {
    fn from(h: Header) -> Self {
        ExecutorContext {
            block_number:           h.number.into(),
            block_hash:             Hasher::digest(h.encode().unwrap()),
            block_coinbase:         h.proposer,
            block_timestamp:        h.timestamp.into(),
            chain_id:               h.chain_id.into(),
            difficulty:             h.difficulty,
            origin:                 h.proposer,
            gas_price:              Default::default(),
            block_gas_limit:        h.gas_limit,
            block_base_fee_per_gas: h.base_fee_per_gas,
            logs:                   Vec::new(),
        }
    }
}
