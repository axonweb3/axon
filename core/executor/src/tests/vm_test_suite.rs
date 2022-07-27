use crate::debugger::{clear_data, EvmDebugger};
use crate::system::SystemExecutor;
use crate::{debugger, AxonExecutor, AxonExecutorAdapter};
use ethers_core::utils::hex;
use evm::{Config, ExitReason, ExitSucceed};
use protocol::types::{ExecutorContext, MemoryAccount, MemoryVicinity, SignedTransaction};
use protocol::{
    traits::Executor,
    types::{MemoryBackend, H160, H256, U256},
};
use serde::{Deserialize, Deserializer};
use std::sync::Arc;
use std::{collections::BTreeMap, io::BufReader, mem::size_of, str::FromStr};

use super::vm_state::{mock_signed_tx, AccountState};
use super::{gen_vicinity, vm_state};

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
        // println!("drop resource");
        clear_data(DEBUG_PATH);
    }
}

impl vm_state::TestEvmState for VmStateDebugger {
    fn init_state() -> Self {
        // println!("888888init_state8888888888888");
        VmStateDebugger {
            // vicinity: gen_vicinity(),
            // state: BTreeMap::new(),
            config:   Config::london(),
            debugger: EvmDebugger::new_empty(DEBUG_PATH),
            executor: AxonExecutor::default(),
            exec_ctx: ExecutorContext::default(),
            txs:      Vec::new(),
        }
    }

    // fn try_apply_chain_id(mut self, id: U256) -> Result<Self, String> {
    //     // println!("888888try_apply_chain_id8888888888888");
    //     // self.vicinity.chain_id = id;
    //     Ok(self)
    // }

    fn try_apply_network_type(mut self, net_type: vm_state::NetworkType) -> Result<Self, String> {
        // println!("888888888try_apply_network_type8888888888");
        match net_type {
            vm_state::NetworkType::Berlin => self.config = Config::berlin(),
            vm_state::NetworkType::Istanbul => self.config = Config::istanbul(),
            vm_state::NetworkType::London => self.config = Config::london(),
        }
        Ok(self)
    }

    // pre
    fn try_apply_accounts<I>(mut self, iter: I) -> Result<Self, String>
    where
        I: Iterator<Item = (H160, AccountState)>,
    {
        // println!("8888888try_apply_accounts888888888888");
        let states: BTreeMap<H160, MemoryAccount> =
            iter.map(|(k, v)| (k, v.try_into().unwrap())).collect();
        self.debugger.set_state_root(states.into_iter());
        Ok(self)
    }

    // blocks-blockheader
    fn try_apply_block_header(mut self, header: vm_state::BlockHeader) -> Result<Self, String> {
        // println!("8888888try_apply_block_header888888888888");
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
        // println!("888888888try_apply_transaction8888888888");
        self.txs = vec![mock_signed_tx(transaction)];
        // let tx = mock_signed_tx(transaction);
        // let mut backend = MemoryBackend::new(&self.vicinity, self.state);
        // // self.executor.exec(&mut backend, vec![tx]);
        let res = self.debugger.exec_with_ectx(
            self.exec_ctx.clone(),
            self.txs.clone(),
            self.config.clone(),
        );
        // println!("res: {:#?}", res);
        _ = res; // todo
        Ok(self)
    }

    // post
    fn validate_account(
        &self,
        address: H160,
        account_state: vm_state::AccountState,
    ) -> Result<(), String> {
        let backend = AxonExecutorAdapter::from_root(
            self.debugger.get_state_root(),
            Arc::clone(&self.debugger.get_trie_db()),
            Arc::clone(&self.debugger.get_storage()),
            self.exec_ctx.clone(),
        )
        .unwrap();
        let account = self.executor.get_account(&backend, &address);
        // println!("get_account================== ");
        // println!("{:#?}", account);
        if account.balance != account_state.balance {
            // println!("get_account================== ");
            // println!("{:#?}", account);
            // println!("=================failed====");
            // println!("====failed:{:?}", address);
            // println!("failed:");
            Err(format!(
                "failed: test case mismatch,
                address: {:?},
                current: {{ balance: {}, nonce: {}}},
                expected: {{ balance: {}, nonce: {}}}",
                address,
                account.balance,
                account.nonce,
                // account.code_hash,  // , code: {}
                // account.storage_root,
                account_state.balance,
                account_state.nonce,
                // hex::encode(account_state.code), // , code: {}
                // account_state.storage,
            ))
            //, storage: {}
        } else {
            println!("pass: {:?}", address);
            // println!("=================ok====");
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn run_tests() {
        vm_state::run_evm_tests::<VmStateDebugger>();
    }
}
