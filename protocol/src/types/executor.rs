pub use ethereum::{AccessList, AccessListItem, Account};
pub use evm::{Config, ExitReason};

use crate::types::{H160, H256, U256};

#[derive(Clone, Debug)]
pub struct ExecResponse {
    pub exit_reason: ExitReason,
    pub ret:         Vec<u8>,
    pub remain_gas:  u64,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ExecutorContext {
    pub block_number:           U256,
    pub block_hash:             H256,
    pub block_coinbase:         H160,
    pub block_timestamp:        U256,
    pub chain_id:               U256,
    pub difficulty:             U256,
    pub origin:                 H160,
    pub gas_price:              U256,
    pub block_gas_limit:        U256,
    pub block_base_fee_per_gas: U256,
}
