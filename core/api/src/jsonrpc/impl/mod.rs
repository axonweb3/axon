mod axon;
mod ckb_light_client;
mod filter;
mod node;
mod web3;

use protocol::types::U256;

use crate::jsonrpc::error::RpcError;

pub use axon::AxonRpcImpl;
pub use ckb_light_client::CkbLightClientRpcImpl;
pub use filter::filter_module;
pub use node::NodeRpcImpl;
pub use web3::{from_receipt_to_web3_log, Web3RpcImpl};

#[allow(clippy::result_large_err)]
pub(crate) fn u256_cast_u64(value: U256) -> Result<u64, RpcError> {
    if value > u64::max_value().into() {
        Err(RpcError::InvalidRequestParams(value))
    } else {
        Ok(value.as_u64())
    }
}
