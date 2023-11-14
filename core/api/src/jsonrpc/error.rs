use jsonrpsee::types::{error::ErrorObject, ErrorObjectOwned};

use protocol::types::{ExitReason, TxResp, U256};
use protocol::{codec::hex_encode, Display};

use core_executor::decode_revert_msg;

use crate::jsonrpc::web3_types::BlockId;

#[derive(Clone, Display, Debug)]
pub enum RpcError {
    #[display(fmt = "Decode interoperation signature r error since {}", _0)]
    DecodeInteroperationSigR(String),
    #[display(fmt = "Decode interoperation signature s error since {}", _0)]
    DecodeInteroperationSigS(String),
    #[display(fmt = "Invalid address source")]
    InvalidAddressSource,
    #[display(fmt = "Missing dummy input cell")]
    MissingDummyInputCell,
    #[display(fmt = "Cannot find image cell")]
    CannotFindImageCell,
    #[display(fmt = "Gas price is zero")]
    GasPriceIsZero,
    #[display(fmt = "Gas price is too large")]
    GasPriceIsTooLarge,
    #[display(fmt = "Gas limit is less than 21000")]
    GasLimitIsTooLow,
    #[display(fmt = "Gas limit is too large")]
    GasLimitIsTooLarge,
    #[display(fmt = "Gas limit is zero")]
    GasLimitIsZero,
    #[display(fmt = "Transaction is not signed")]
    TransactionIsNotSigned,
    #[display(fmt = "Cannot get latest block")]
    CannotGetLatestBlock,
    #[display(fmt = "Invalid block hash")]
    InvalidBlockHash,
    #[display(fmt = "Invalid from block number {}", _0)]
    InvalidFromBlockNumber(u64),
    #[display(fmt = "Invalid block range from {} to {} limit to {}", _0, _1, _2)]
    InvalidBlockRange(u64, u64, u64),
    #[display(fmt = "Invalid newest block {:?}", _0)]
    InvalidNewestBlock(BlockId),
    #[display(fmt = "Invalid position {}", _0)]
    InvalidPosition(U256),
    #[display(fmt = "Cannot find the block")]
    CannotFindBlock,
    #[display(fmt = "Invalid reward percentiles {} {}", _0, _1)]
    InvalidRewardPercentiles(f64, f64),
    #[display(fmt = "Invalid from block number and to block number union")]
    InvalidFromBlockAndToBlockUnion,
    #[display(fmt = "Invalid filter id {}", _0)]
    CannotFindFilterId(u64),

    #[display(fmt = "EVM error {}", "decode_revert_msg(&_0.ret)")]
    Evm(TxResp),
    #[display(fmt = "Internal error: {}", _0)]
    Internal(String),
}

impl From<RpcError> for String {
    fn from(err: RpcError) -> Self {
        err.to_string()
    }
}

impl RpcError {
    fn code(&self) -> i32 {
        match self {
            RpcError::DecodeInteroperationSigR(_) => -40001,
            RpcError::DecodeInteroperationSigS(_) => -40002,
            RpcError::InvalidAddressSource => -40003,
            RpcError::MissingDummyInputCell => -40004,
            RpcError::CannotFindImageCell => -40005,
            RpcError::GasPriceIsZero => -40006,
            RpcError::GasPriceIsTooLarge => -40007,
            RpcError::GasLimitIsTooLow => -40008,
            RpcError::GasLimitIsTooLarge => -40009,
            RpcError::GasLimitIsZero => -40010,
            RpcError::TransactionIsNotSigned => -40011,
            RpcError::CannotGetLatestBlock => -40013,
            RpcError::InvalidBlockHash => -40014,
            RpcError::InvalidFromBlockNumber(_) => -40015,
            RpcError::InvalidBlockRange(_, _, _) => -40016,
            RpcError::InvalidNewestBlock(_) => -40017,
            RpcError::InvalidPosition(_) => -40018,
            RpcError::CannotFindBlock => -40019,
            RpcError::InvalidRewardPercentiles(_, _) => -40020,
            RpcError::InvalidFromBlockAndToBlockUnion => -40021,
            RpcError::CannotFindFilterId(_) => -40022,

            RpcError::Evm(_) => -49998,
            RpcError::Internal(_) => -49999,
        }
    }
}

impl From<RpcError> for ErrorObjectOwned {
    fn from(err: RpcError) -> Self {
        let none_data: Option<String> = None;
        let err_code = err.code();
        match &err {
            RpcError::DecodeInteroperationSigR(msg) => {
                ErrorObject::owned(err_code, err.clone(), Some(msg))
            }
            RpcError::DecodeInteroperationSigS(msg) => {
                ErrorObject::owned(err_code, err.clone(), Some(msg))
            }
            RpcError::InvalidAddressSource => ErrorObject::owned(err_code, err, none_data),
            RpcError::MissingDummyInputCell => ErrorObject::owned(err_code, err, none_data),
            RpcError::CannotFindImageCell => ErrorObject::owned(err_code, err, none_data),
            RpcError::GasPriceIsZero => ErrorObject::owned(err_code, err, none_data),
            RpcError::GasPriceIsTooLarge => ErrorObject::owned(err_code, err, none_data),
            RpcError::GasLimitIsTooLow => ErrorObject::owned(err_code, err, none_data),
            RpcError::GasLimitIsTooLarge => ErrorObject::owned(err_code, err, none_data),
            RpcError::GasLimitIsZero => ErrorObject::owned(err_code, err, none_data),
            RpcError::TransactionIsNotSigned => ErrorObject::owned(err_code, err, none_data),
            RpcError::CannotGetLatestBlock => ErrorObject::owned(err_code, err, none_data),
            RpcError::InvalidBlockHash => ErrorObject::owned(err_code, err, none_data),
            RpcError::InvalidFromBlockNumber(_) => ErrorObject::owned(err_code, err, none_data),
            RpcError::InvalidBlockRange(_, _, _) => ErrorObject::owned(err_code, err, none_data),
            RpcError::InvalidNewestBlock(_) => ErrorObject::owned(err_code, err, none_data),
            RpcError::InvalidPosition(_) => ErrorObject::owned(err_code, err, none_data),
            RpcError::CannotFindBlock => ErrorObject::owned(err_code, err, none_data),
            RpcError::InvalidRewardPercentiles(_, _) => {
                ErrorObject::owned(err_code, err, none_data)
            }
            RpcError::InvalidFromBlockAndToBlockUnion => {
                ErrorObject::owned(err_code, err, none_data)
            }
            RpcError::CannotFindFilterId(_) => ErrorObject::owned(err_code, err, none_data),

            RpcError::Evm(resp) => {
                ErrorObject::owned(err_code, err.clone(), Some(vm_err(resp.clone())))
            }
            RpcError::Internal(msg) => ErrorObject::owned(err_code, err.clone(), Some(msg)),
        }
    }
}

pub fn vm_err(resp: TxResp) -> String {
    match resp.exit_reason {
        ExitReason::Revert(_) => format!("0x{}", hex_encode(&resp.ret),),
        ExitReason::Error(err) => format!("{:?}", err),
        ExitReason::Fatal(fatal) => format!("{:?}", fatal),
        _ => unreachable!(),
    }
}
