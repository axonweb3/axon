use crate::types::{H256, U256};

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ExecutorContext {
    pub block_number:           U256,
    pub block_hash:             H256,
    pub block_coinbase:         U256,
    pub block_timestamp:        U256,
    pub chain_id:               U256,
    pub difficulty:             U256,
    pub origin:                 H256,
    pub gas_price:              U256,
    pub block_gas_limit:        U256,
    pub block_base_fee_per_gas: U256,
}
