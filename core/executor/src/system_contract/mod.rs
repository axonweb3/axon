mod error;
pub mod image_cell;
pub mod metadata;
mod native_token;

pub use crate::system_contract::image_cell::ImageCellContract;
pub use crate::system_contract::metadata::MetadataContract;
pub use crate::system_contract::native_token::NativeTokenContract;

use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{ExitReason, ExitRevert, ExitSucceed, SignedTransaction, TxResp, H160, U256};

#[macro_export]
macro_rules! exec_try {
    ($func: expr, $gas_limit: expr, $log_msg: literal) => {
        match $func {
            Ok(r) => r,
            Err(e) => {
                log::error!("{:?} {:?}", $log_msg, e);
                return $crate::system_contract::revert_resp($gas_limit);
            }
        }
    };
}

pub const fn system_contract_address(addr: u8) -> H160 {
    H160([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, addr,
    ])
}

pub trait SystemContract {
    const ADDRESS: H160;

    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp;
}

pub fn system_contract_dispatch<B: Backend + ApplyBackend>(
    backend: &mut B,
    tx: &SignedTransaction,
) -> Option<TxResp> {
    if let Some(addr) = tx.get_to() {
        if addr == NativeTokenContract::ADDRESS {
            return Some(NativeTokenContract::default().exec_(backend, tx));
        } else if addr == MetadataContract::ADDRESS {
            return Some(MetadataContract::default().exec_(backend, tx));
        } else if addr == ImageCellContract::ADDRESS {
            return Some(ImageCellContract::default().exec_(backend, tx));
        }
    }

    None
}

pub fn revert_resp(gas_limit: U256) -> TxResp {
    TxResp {
        exit_reason:  ExitReason::Revert(ExitRevert::Reverted),
        ret:          vec![],
        gas_used:     (gas_limit - 1).as_u64(),
        remain_gas:   1u64,
        fee_cost:     U256::one(),
        logs:         vec![],
        code_address: None,
        removed:      false,
    }
}

pub fn succeed_resp(gas_limit: U256) -> TxResp {
    TxResp {
        exit_reason:  ExitReason::Succeed(ExitSucceed::Returned),
        ret:          vec![],
        gas_used:     1u64,
        remain_gas:   (gas_limit - 1).as_u64(),
        fee_cost:     U256::one(),
        logs:         vec![],
        code_address: None,
        removed:      false,
    }
}
