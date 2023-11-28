use crate::types::U64;

pub const MAX_GAS_LIMIT: u64 = 30_000_000;
pub const MAX_GAS_PRICE: U64 = U64([u64::MAX / MAX_GAS_LIMIT / 100_000 - 1]);
pub const MIN_TRANSACTION_GAS_LIMIT: u64 = 21_000;
pub const MEMPOOL_REFRESH_TIMEOUT: u64 = 50;
pub const MAX_BLOCK_GAS_LIMIT: u64 = 30_000_000;
// MAX_FEE_HISTORY is the maximum number of blocks that can be retrieved for a
// fee history request. Between 1 and 1024 blocks can be requested in a single
// query. reference: https://docs.infura.io/infura/networks/ethereum/json-rpc-methods/eth_feehistory/
pub const MAX_FEE_HISTORY: u64 = 1024;
pub const MAX_RPC_GAS_CAP: u64 = 50_000_000;
pub const BASE_FEE_PER_GAS: u64 = 0x539;
