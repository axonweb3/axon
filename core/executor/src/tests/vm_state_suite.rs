use crate::debugger::{clear_data, EvmDebugger};
use crate::{AxonExecutor, AxonExecutorAdapter};
use evm::Config;
use protocol::types::{ExecutorContext, MemoryAccount, SignedTransaction, U256};
use protocol::{traits::Executor, types::H160};
use std::collections::BTreeMap;
use std::sync::Arc;

use super::vm_state;
use super::vm_state::{mock_signed_tx, AccountState};

const DEBUG_PATH: &str = "../../devtools/chain/db3";

struct VmStateDebugger {
    config:   Config,
    debugger: EvmDebugger,
    executor: AxonExecutor,
    exec_ctx: ExecutorContext,
    txs:      Vec<SignedTransaction>,
}

impl Drop for VmStateDebugger {
    fn drop(&mut self) {
        clear_data(DEBUG_PATH);
    }
}

impl vm_state::TestEvmState for VmStateDebugger {
    fn init_state() -> Self {
        VmStateDebugger {
            config:   Config::london(),
            debugger: EvmDebugger::new(vec![], U256::zero(), DEBUG_PATH),
            executor: AxonExecutor::default(),
            exec_ctx: ExecutorContext::default(),
            txs:      Vec::new(),
        }
    }

    fn try_apply_network_type(mut self, net_type: vm_state::NetworkType) -> Result<Self, String> {
        match net_type {
            vm_state::NetworkType::Berlin => self.config = Config::berlin(),
            vm_state::NetworkType::Istanbul => self.config = Config::istanbul(),
            vm_state::NetworkType::London => self.config = Config::london(),
            vm_state::NetworkType::Merge => self.config = Config::london(), // todo
        }
        Ok(self)
    }

    // pre
    fn try_apply_accounts<I>(mut self, iter: I) -> Result<Self, String>
    where
        I: Iterator<Item = (H160, AccountState)>,
    {
        self.debugger.exec(0, vec![]);
        let states: BTreeMap<H160, MemoryAccount> =
            iter.map(|(k, v)| (k, v.try_into().unwrap())).collect();
        self.debugger.set_state_root(states.into_iter());
        Ok(self)
    }

    // blocks-blockheader
    fn try_apply_block_header(mut self, header: &vm_state::BlockHeader) -> Result<Self, String> {
        self.exec_ctx.block_coinbase = header.coinbase;
        self.exec_ctx.difficulty = header.difficulty;
        self.exec_ctx.block_gas_limit = header.gas_limit;
        self.exec_ctx.block_hash = header.hash;
        self.exec_ctx.block_number = header.number;
        self.exec_ctx.block_timestamp = header.timestamp;
        Ok(self)
    }

    // blocks-transaction
    fn try_apply_transaction(
        mut self,
        transaction: vm_state::CallTransaction,
    ) -> Result<Self, String> {
        self.txs = vec![mock_signed_tx(transaction)];
        let res = self.debugger.exec_with_ectx(
            self.exec_ctx.clone(),
            self.txs.clone(),
            self.config.clone(),
        );
        _ = res;
        // todo use
        // println!("{:#?}", res);
        Ok(self)
    }

    // post
    fn validate_account(
        &self,
        address: H160,
        coinbase: H160,
        account_state: vm_state::AccountState,
    ) -> Result<(), String> {
        let backend = AxonExecutorAdapter::from_root(
            self.debugger.state_root(),
            Arc::clone(&self.debugger.trie_db()),
            Arc::clone(&self.debugger.storage()),
            self.exec_ctx.clone(),
        )
        .unwrap();
        let account = self.executor.get_account(&backend, &address);
        if account.balance != account_state.balance && address != coinbase {
            Err(format!(
                "failed: test case mismatch,
                address: {:?},
                current:  {{ balance: {}, nonce: {}}},
                expected: {{ balance: {}, nonce: {}}}",
                address, account.balance, account.nonce, account_state.balance, account_state.nonce,
            ))
        } else {
            println!("pass: {:?}", address);
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tests::vm_state::print_result;

    use super::*;
    #[test]
    fn run_tests() {
        vm_state::run_evm_tests::<VmStateDebugger>();
    }

    #[test]
    fn run_single_test() {
        let num = vm_state::run_evm_test::<VmStateDebugger>(
            vm_state::BLOCK_INFO,
            "blockInfo_d0g0v0_London",
        );
        print_result(num);
    }
}
