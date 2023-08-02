use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{Apply, Basic, SignedTransaction, TxResp, H160, U256};

use crate::system_contract::utils::{revert_resp, succeed_resp};
use crate::system_contract::{system_contract_address, SystemContract};

pub const NATIVE_TOKEN_CONTRACT_ADDRESS: H160 = system_contract_address(0x0);

#[derive(Default)]
pub struct NativeTokenContract;

impl SystemContract for NativeTokenContract {
    const ADDRESS: H160 = NATIVE_TOKEN_CONTRACT_ADDRESS;

    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp {
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();
        let tx_value = *tx.value();
        let gas_limit = *tx.gas_limit();

        if tx_data.len() < 21 || tx_data[0] > 1 {
            return revert_resp(gas_limit);
        }

        let direction = tx_data[0] == 0u8;
        let l2_addr = H160::from_slice(&tx_data[1..21]);
        let mut account = backend.basic(l2_addr);

        if direction {
            account.balance += tx_value;
        } else {
            if account.balance < tx_value {
                return revert_resp(gas_limit);
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

        succeed_resp(gas_limit)
    }
}
