#![feature(test)]

use std::collections::BTreeMap;

use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};

use protocol::traits::{ApplyBackend, Backend, Executor};
use protocol::types::{
    Address, Config, ExecResp, SignedTransaction, TransactionAction, H256, U256,
};

pub mod adapter;

#[derive(Default)]
pub struct EvmExecutor;

impl EvmExecutor {
    pub fn new() -> Self {
        EvmExecutor::default()
    }
}

impl Executor for EvmExecutor {
    // Used for query data API, this function will not modify the world state.
    fn call<B: Backend>(&self, backend: &mut B, addr: Address, data: Vec<u8>) -> ExecResp {
        let config = Config::london();
        let metadata = StackSubstateMetadata::new(u64::MAX, &config);
        let state = MemoryStackState::new(metadata, backend);
        let precompiles = BTreeMap::new();
        let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);
        let (exit_reason, ret) = executor.transact_call(
            Address::default(),
            addr,
            U256::default(),
            data,
            u64::MAX,
            Vec::new(),
        );

        ExecResp {
            exit_reason,
            ret,
            remain_gas: 0,
            gas_used: 0,
            logs: vec![],
        }
    }

    // Function execute returns exit_reason, ret_data and remain_gas.
    fn exec<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: SignedTransaction) -> ExecResp {
        let config = Config::london();
        let metadata = StackSubstateMetadata::new(u64::MAX, &config);
        let state = MemoryStackState::new(metadata, backend);
        let precompiles = BTreeMap::new();
        let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

        let (exit_reason, ret) = match tx.transaction.unsigned.action {
            TransactionAction::Call(addr) => executor.transact_call(
                tx.sender,
                addr,
                tx.transaction.unsigned.value,
                tx.transaction.unsigned.input,
                tx.transaction.unsigned.gas_limit.as_u64(),
                tx.transaction
                    .unsigned
                    .access_list
                    .iter()
                    .map(|x| (x.address, x.slots.clone()))
                    .collect(),
            ),
            TransactionAction::Create => {
                let exit_reason = executor.transact_create2(
                    tx.sender,
                    tx.transaction.unsigned.value,
                    tx.transaction.unsigned.input,
                    H256::default(),
                    tx.transaction.unsigned.gas_limit.as_u64(),
                    tx.transaction
                        .unsigned
                        .access_list
                        .iter()
                        .map(|x| (x.address, x.slots.clone()))
                        .collect(),
                );
                (exit_reason, Vec::new())
            }
        };
        let remain_gas = executor.gas();
        let gas_used = executor.used_gas();

        if exit_reason.is_succeed() {
            let (values, logs) = executor.into_state().deconstruct();
            backend.apply(values, logs, true);
        }

        ExecResp {
            exit_reason,
            ret,
            remain_gas,
            gas_used,
            logs: vec![],
        }
    }
}
