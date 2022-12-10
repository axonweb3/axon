use protocol::codec::ProtocolCodec;
use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{
    Apply, Basic, ExitReason, ExitRevert, ExitSucceed, SignedTransaction, TxResp, H160, U256,
};

use crate::system_contract::{system_contract_address, SystemContract};

#[derive(Default)]
pub struct NativeTokenContract;

impl SystemContract for NativeTokenContract {
    const ADDRESS: H160 = system_contract_address(0x0);

    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp {
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();
        let tx_value = *tx.value();

        if tx_data.len() < 21 || tx_data[0] > 1 {
            return revert_resp(*tx.gas_limit());
        }

        let direction = tx_data[0] == 0u8;
        let l2_addr = H160::from_slice(&tx_data[1..21]);
        let mut account = backend.basic(l2_addr);

        if direction {
            account.balance += tx_value;
        } else {
            if account.balance < tx_value {
                return revert_resp(*tx.gas_limit());
            }

            account.balance -= tx_value;
        }

        backend.apply(
            vec![Apply::Modify {
                address:       l2_addr,
                basic:         Basic {
                    balance: account.balance,
                    nonce:   account.nonce + U256::one(),
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
            gas_used:     0u64,
            remain_gas:   tx.gas_limit().as_u64(),
            fee_cost:     U256::zero(),
            logs:         vec![],
            code_address: None,
            removed:      false,
        }
    }
}

fn revert_resp(gas_limit: U256) -> TxResp {
    TxResp {
        exit_reason:  ExitReason::Revert(ExitRevert::Reverted),
        ret:          vec![],
        gas_used:     1u64,
        remain_gas:   (gas_limit - 1).as_u64(),
        fee_cost:     U256::one(),
        logs:         vec![],
        code_address: None,
        removed:      false,
    }
}
