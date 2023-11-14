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

const MAX_U64: U256 = U256([u64::MAX, 0, 0, 0]);

#[allow(clippy::result_large_err)]
pub(crate) fn u256_cast_u64(value: U256) -> Result<u64, RpcError> {
    if value > MAX_U64 {
        Err(RpcError::InvalidRequestParams(value))
    } else {
        Ok(value.low_u64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u256_cast_u64() {
        assert_eq!(u256_cast_u64(U256::zero()).ok(), Some(0u64));
        assert_eq!(u256_cast_u64(MAX_U64).ok(), Some(u64::MAX));
        assert!(u256_cast_u64(u128::MAX.into()).is_err());
    }
}
