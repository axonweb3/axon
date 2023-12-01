use crate::types::U64;

/// There is not a standard for the maximum gas limit, as long as the account
/// balance can pay the `gas_limit * gas_price`. For reduce some useless
/// calculation, `30_000_000` is large enough to cover the transaction usage.
pub const MAX_GAS_LIMIT: u64 = 30_000_000;
/// According to [go-ethereum](https://github.com/ethereum/go-ethereum/blob/be65b47/eth/gasprice/gasprice.go#L38),
/// the maximum gas price is 500 Gwei.
pub const MAX_GAS_PRICE: U64 = U64([500 * GWEI]);
pub const MIN_TRANSACTION_GAS_LIMIT: u64 = 21_000;
/// The mempool refresh timeout is 50 milliseconds.
pub const MEMPOOL_REFRESH_TIMEOUT: u64 = 50;
pub const MAX_BLOCK_GAS_LIMIT: u64 = 30_000_000;
// MAX_FEE_HISTORY is the maximum number of blocks that can be retrieved for a
// fee history request. Between 1 and 1024 blocks can be requested in a single
// query. reference: https://docs.infura.io/infura/networks/ethereum/json-rpc-methods/eth_feehistory/
pub const MAX_FEE_HISTORY: u64 = 1024;
pub const MAX_RPC_GAS_CAP: u64 = 50_000_000;
pub const BASE_FEE_PER_GAS: u64 = 0x539;

const GWEI: u64 = 1_000_000_000;
