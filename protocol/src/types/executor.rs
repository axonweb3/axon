pub use ethereum::{AccessList, AccessListItem, Account};
pub use evm::{backend::Log, Config, ExitReason};

use crate::types::{Hash, H160, U256};

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
