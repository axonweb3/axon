use crate::debugger::{clear_data, EvmDebugger};
use crate::{AxonExecutor, AxonExecutorAdapter};
use evm::Config;
use protocol::types::{
    Bytes, ExecutorContext, LegacyTransaction, MemoryAccount, SignatureComponents,
    SignedTransaction, TransactionAction, UnsignedTransaction, UnverifiedTransaction, H256, U256,
};
use protocol::{traits::Executor, types::H160};
use std::collections::BTreeMap;
use std::sync::Arc;

use evm_test_suite::block_chain_tests::AccountState;
use evm_test_suite::block_chain_tests::CallTransaction;

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

impl evm_test_suite::block_chain_tests::TestEvmState for VmStateDebugger {
    fn init_state() -> Self {
        VmStateDebugger {
            config:   Config::london(),
            debugger: EvmDebugger::new(Vec::new(), U256::zero(), DEBUG_PATH),
            executor: AxonExecutor::default(),
            exec_ctx: ExecutorContext::default(),
            txs:      Vec::new(),
        }
    }

    fn try_apply_network_type(
        mut self,
        net_type: evm_test_suite::block_chain_tests::NetworkType,
    ) -> Result<Self, String> {
        match net_type {
            evm_test_suite::block_chain_tests::NetworkType::Berlin => {
                self.config = Config::berlin()
            }
            evm_test_suite::block_chain_tests::NetworkType::Istanbul => {
                self.config = Config::istanbul()
            }
            evm_test_suite::block_chain_tests::NetworkType::London => {
                self.config = Config::london()
            }
            evm_test_suite::block_chain_tests::NetworkType::Merge => self.config = Config::london(), /* todo */
        }
        Ok(self)
    }

    // pre
    fn try_apply_accounts<I>(mut self, iter: I) -> Result<Self, String>
    where
        I: Iterator<Item = (H160, AccountState)>,
    {
        self.debugger.exec(0, Vec::new());
        let states: BTreeMap<H160, MemoryAccount> =
            iter.map(|(k, v)| (k, v.try_into().unwrap())).collect();
        self.debugger.set_state_root(states.into_iter());
        Ok(self)
    }

    // blocks-blockheader
    fn try_apply_block_header(
        mut self,
        header: &evm_test_suite::block_chain_tests::BlockHeader,
    ) -> Result<Self, String> {
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
        transaction: evm_test_suite::block_chain_tests::CallTransaction,
    ) -> Result<Self, String> {
        self.txs = vec![mock_signed_tx(transaction)];
        let _ = self.debugger.exec_with_ectx(
            self.exec_ctx.clone(),
            self.txs.clone(),
            self.config.clone(),
        );
        Ok(self)
    }

    // post
    fn validate_account(
        &self,
        address: H160,
        coinbase: H160,
        skip_coinbase: bool,
        account_state: evm_test_suite::block_chain_tests::AccountState,
    ) -> Result<(), String> {
        let backend = AxonExecutorAdapter::from_root(
            self.debugger.state_root(),
            Arc::clone(&self.debugger.trie_db()),
            Arc::clone(&self.debugger.storage()),
            self.exec_ctx.clone(),
        )
        .unwrap();
        let account = self.executor.get_account(&backend, &address);
        if skip_coinbase && address == coinbase {
            println!("skip: {:?}", address);
            return Err(format!("skip coinbase"));
        }
        if account.balance != account_state.balance {
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

fn mock_signed_tx(tx: CallTransaction) -> SignedTransaction {
    let utx = UnverifiedTransaction {
        unsigned:  UnsignedTransaction::Legacy(LegacyTransaction {
            nonce:     tx.nonce,
            gas_limit: tx.gas_limit,
            gas_price: tx.gas_price,
            action:    TransactionAction::Call(tx.to),
            value:     tx.value,
            data:      Bytes::copy_from_slice(&tx.data),
        }),
        chain_id:  5u64,
        hash:      H256::default(),
        signature: Some(SignatureComponents {
            standard_v: tx.v[0],
            r:          Bytes::copy_from_slice(&tx.r),
            s:          Bytes::copy_from_slice(&tx.s),
        }),
    }
    .calc_hash();
    SignedTransaction {
        transaction: utx,
        sender:      tx.sender,
        public:      None,
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn run_tests() {
        evm_test_suite::block_chain_tests::run_tests::<VmStateDebugger>(true);
    }

    #[test]
    fn run_single_file() {
        evm_test_suite::block_chain_tests::run_single_test::<VmStateDebugger>(
            evm_test_suite::block_chain_tests::vm::BLOCK_INFO,
            "",
            true,
        );
    }

    #[test]
    fn run_single_test() {
        evm_test_suite::block_chain_tests::run_single_test::<VmStateDebugger>(
            evm_test_suite::block_chain_tests::vm::BLOCK_INFO,
            "blockInfo_d0g0v0_London",
            true,
        );
    }
}
