use async_trait::async_trait;
use ethereum::TransactionAction;
use evm::backend::{ApplyBackend, Backend};
use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use evm::Config;
use protocol::traits::{ExecuteResult, Executor as ExecutorT};
use protocol::types::{Address, SignedTransaction, H256, U256};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Executor {}

impl Executor {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ExecutorT for Executor {
    // Used for query data API, this function will not modify the world state.
    async fn call<B: Backend + Send>(
        &self,
        backend: &mut B,
        addr: Address,
        data: Vec<u8>,
    ) -> ExecuteResult {
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
        return ExecuteResult {
            exit_reason,
            ret,
            remain_gas: 0,
        };
    }

    // Function execute returns exit_reason, ret_data and remain_gas.
    async fn execute<B: Backend + ApplyBackend + Send>(
        &self,
        backend: &mut B,
        tx: SignedTransaction,
    ) -> ExecuteResult {
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
        let gas = executor.gas();

        if exit_reason.is_succeed() {
            let (values, logs) = executor.into_state().deconstruct();
            backend.apply(values, logs, true);
        }

        ExecuteResult {
            exit_reason,
            ret,
            remain_gas: gas,
        }
    }
}
