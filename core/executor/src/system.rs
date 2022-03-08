use evm::{ExitRevert, ExitSucceed};

use protocol::codec::ProtocolCodec;
use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{
    Apply, Basic, ExitReason, SignedTransaction, TransactionAction, TxResp, H160, U256,
};

pub const NATIVE_TOKEN_ISSUE_ADDRESS: H160 = H160::zero();

#[derive(Default)]
pub struct SystemExecutor;

impl SystemExecutor {
    pub fn new() -> Self {
        SystemExecutor::default()
    }

    pub(crate) fn inner_exec<B: Backend + ApplyBackend>(
        &self,
        backend: &mut B,
        tx: SignedTransaction,
    ) -> TxResp {
        match classify_script(&tx.transaction.unsigned.action) {
            SystemScriptCategory::NativeToken => call_native_token(backend, tx),
        }
    }
}

enum SystemScriptCategory {
    NativeToken,
}

fn call_native_token<B: Backend + ApplyBackend>(backend: &mut B, tx: SignedTransaction) -> TxResp {
    let tx = tx.transaction.unsigned;

    if tx.data.len() < 21 || tx.data[0] > 1 {
        return revert_resp(tx.gas_limit);
    }

    let direction = tx.data[0] == 0u8;
    let l2_addr = H160::from_slice(&tx.data[1..21]);
    let mut account = backend.basic(l2_addr);

    if direction {
        account.balance += tx.value;
    } else {
        if account.balance < tx.value {
            return revert_resp(tx.gas_limit);
        }

        account.balance -= tx.value;
    }

    backend.apply(
        vec![Apply::Modify {
            address:       l2_addr,
            basic:         Basic {
                balance: account.balance,
                nonce:   account.nonce,
            },
            code:          None,
            storage:       vec![],
            reset_storage: false,
        }],
        vec![],
        false,
    );

    TxResp {
        exit_reason:  ExitReason::Succeed(ExitSucceed::Returned),
        ret:          account.balance.encode().unwrap().to_vec(),
        gas_used:     1u64,
        remain_gas:   (tx.gas_limit - 1u64).as_u64(),
        logs:         vec![],
        code_address: None,
    }
}

fn classify_script(action: &TransactionAction) -> SystemScriptCategory {
    match action {
        TransactionAction::Call(_addr) => SystemScriptCategory::NativeToken,
        TransactionAction::Create => unreachable!(),
    }
}

fn revert_resp(gas_limit: U256) -> TxResp {
    TxResp {
        exit_reason:  ExitReason::Revert(ExitRevert::Reverted),
        ret:          vec![],
        gas_used:     1u64,
        remain_gas:   (gas_limit - 1).as_u64(),
        logs:         vec![],
        code_address: None,
    }
}
