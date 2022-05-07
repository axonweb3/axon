use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};

use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{
    Config, Hasher, SignedTransaction, TransactionAction, TxResp, H160, H256, U256,
};

use crate::precompiles::build_precompile_set;

#[derive(Default)]
pub struct EvmExecutor;

impl EvmExecutor {
    pub fn new() -> Self {
        EvmExecutor::default()
    }

    pub fn inner_exec<B: Backend + ApplyBackend>(
        &self,
        backend: &mut B,
        tx: SignedTransaction,
    ) -> TxResp {
        let old_nonce = backend.basic(tx.sender).nonce;
        let config = Config::london();
        let metadata = StackSubstateMetadata::new(u64::MAX, &config);
        let state = MemoryStackState::new(metadata, backend);
        let precompiles = build_precompile_set();
        let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);
        let (exit_reason, ret) = match tx.transaction.unsigned.action {
            TransactionAction::Call(addr) => executor.transact_call(
                tx.sender,
                addr,
                tx.transaction.unsigned.value,
                tx.transaction.unsigned.data.to_vec(),
                tx.transaction.unsigned.gas_limit.as_u64(),
                tx.transaction
                    .unsigned
                    .access_list
                    .into_iter()
                    .map(|x| (x.address, x.storage_keys))
                    .collect(),
            ),
            TransactionAction::Create => executor.transact_create(
                tx.sender,
                tx.transaction.unsigned.value,
                tx.transaction.unsigned.data.to_vec(),
                tx.transaction.unsigned.gas_limit.as_u64(),
                tx.transaction
                    .unsigned
                    .access_list
                    .into_iter()
                    .map(|x| (x.address, x.storage_keys))
                    .collect(),
            ),
        };

        let remain_gas = executor.gas();
        let gas_used = executor.used_gas();
        let (values, logs) = executor.into_state().deconstruct();
        backend.apply(values, logs, true);

        let code_address = if exit_reason.is_succeed() {
            if tx.transaction.unsigned.action == TransactionAction::Create {
                Some(code_address(&tx.sender, &old_nonce))
            } else {
                None
            }
        } else {
            None
        };

        TxResp {
            exit_reason,
            ret,
            remain_gas,
            gas_used,
            logs: vec![],
            code_address,
        }
    }
}

pub fn code_address(sender: &H160, nonce: &U256) -> H256 {
    let mut stream = rlp::RlpStream::new_list(2);
    stream.append(sender);
    stream.append(nonce);
    Hasher::digest(&stream.out())
}
