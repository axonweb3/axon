pub use ethereum::{AccessList, AccessListItem, Account};
pub use evm::{backend::Log, Config, ExitReason};

use crate::codec::ProtocolCodec;
use crate::types::{Hash, Hasher, Header, H160, U256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecResp {
    pub exit_reason: ExitReason,
    pub ret:         Vec<u8>,
    pub gas_used:    u64,
    pub remain_gas:  u64,
    pub logs:        Vec<Log>,
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
            difficulty:             h.difficulty.into(),
            origin:                 h.proposer,
            gas_price:              Default::default(),
            block_gas_limit:        h.gas_limit,
            block_base_fee_per_gas: h.base_fee_per_gas.unwrap_or_default(),
            logs:                   Vec::new(),
        }
    }
}
