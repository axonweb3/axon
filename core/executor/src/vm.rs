use std::collections::BTreeMap;

use evm::executor::stack::{MemoryStackState, PrecompileFn, StackExecutor, StackSubstateMetadata};

use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{Config, SignedTransaction, TransactionAction, TxResp, H160};

pub const METADATA_CONTRACT_ADDRESS: H160 = H160([
    176, 13, 97, 107, 130, 12, 57, 97, 158, 226, 158, 81, 68, 208, 34, 108, 248, 181, 193, 90,
]);
pub const WCKB_CONTRACT_ADDRESS: H160 = H160([
    74, 245, 236, 94, 61, 41, 217, 221, 215, 244, 191, 145, 160, 34, 19, 28, 65, 183, 35, 82,
]);
pub const CROSSCHAIN_CONTRACT_ADDRESS: H160 = H160([
    180, 132, 253, 72, 14, 89, 134, 33, 99, 143, 56, 15, 64, 70, 151, 205, 159, 88, 176, 248,
]);

// deprecated
#[allow(dead_code)]
#[derive(Default)]
pub struct EvmExecutor;

#[allow(dead_code)]
impl EvmExecutor {
    pub fn inner_exec<B: Backend + ApplyBackend>(
        &self,
        backend: &mut B,
        config: &Config,
        gas_limit: u64,
        precompiles: &BTreeMap<H160, PrecompileFn>,
        tx: SignedTransaction,
    ) -> TxResp {
        let old_nonce = backend.basic(tx.sender).nonce;
        let metadata = StackSubstateMetadata::new(gas_limit, config);
        let mut executor = StackExecutor::new_with_precompiles(
            MemoryStackState::new(metadata, backend),
            config,
            precompiles,
        );
        let (exit_reason, ret) = match tx.transaction.unsigned.action() {
            TransactionAction::Call(addr) => executor.transact_call(
                tx.sender,
                *addr,
                *tx.transaction.unsigned.value(),
                tx.transaction.unsigned.data().to_vec(),
                gas_limit,
                tx.transaction
                    .unsigned
                    .access_list()
                    .into_iter()
                    .map(|x| (x.address, x.storage_keys))
                    .collect(),
            ),
            TransactionAction::Create => executor.transact_create(
                tx.sender,
                *tx.transaction.unsigned.value(),
                tx.transaction.unsigned.data().to_vec(),
                gas_limit,
                tx.transaction
                    .unsigned
                    .access_list()
                    .into_iter()
                    .map(|x| (x.address, x.storage_keys))
                    .collect(),
            ),
        };

        let remain_gas = executor.gas();
        let gas_used = executor.used_gas();
        let code_address = if tx.transaction.unsigned.action() == &TransactionAction::Create
            && exit_reason.is_succeed()
        {
            Some(crate::code_address(&tx.sender, &old_nonce))
        } else {
            None
        };

        let resp = TxResp {
            exit_reason,
            ret,
            remain_gas,
            gas_used,
            logs: vec![],
            code_address,
            removed: false,
        };

        if resp.exit_reason.is_succeed() {
            let (values, logs) = executor.into_state().deconstruct();
            backend.apply(values, logs, true);
        }

        resp
    }
}
