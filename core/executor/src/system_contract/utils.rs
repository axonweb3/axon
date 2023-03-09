use evm::ExitSucceed;
use protocol::types::{ExitReason, ExitRevert, TxResp, U256};

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
        exit_reason:  ExitReason::Succeed(ExitSucceed::Stopped),
        ret:          vec![],
        gas_used:     0u64,
        remain_gas:   gas_limit.as_u64(),
        fee_cost:     U256::zero(),
        logs:         vec![],
        code_address: None,
        removed:      false,
    }
}
