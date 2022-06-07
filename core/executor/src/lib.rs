#![feature(test)]
#![allow(clippy::derive_partial_eq_without_eq)]

pub mod adapter;
#[cfg(test)]
mod debugger;
mod precompiles;
mod system;
#[cfg(test)]
mod tests;
mod vm;

pub use crate::adapter::{AxonExecutorAdapter, MPTTrie, RocksTrieDB};
pub use crate::{
    system::NATIVE_TOKEN_ISSUE_ADDRESS, vm::code_address, vm::CROSSCHAIN_CONTRACT_ADDRESS,
};

use std::collections::BTreeMap;

use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use evm::CreateScheme;

use common_merkle::Merkle;
use protocol::codec::ProtocolCodec;
use protocol::traits::{ApplyBackend, Backend, Executor, ExecutorAdapter as Adapter};
use protocol::types::{
    Account, Config, ExecResp, Hasher, SignedTransaction, TransactionAction, TxResp, H160,
    NIL_DATA, RLP_NULL, U256,
};

use crate::{system::SystemExecutor, vm::EvmExecutor};

#[derive(Default)]
pub struct AxonExecutor;

impl Executor for AxonExecutor {
    // Used for query data API, this function will not modify the world state.
    fn call<B: Backend>(
        &self,
        backend: &mut B,
        from: Option<H160>,
        to: Option<H160>,
        data: Vec<u8>,
    ) -> TxResp {
        let config = Config::london();
        let metadata = StackSubstateMetadata::new(u64::MAX, &config);
        let state = MemoryStackState::new(metadata, backend);
        let precompiles = BTreeMap::new();
        let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);
        let (exit, res) = if let Some(addr) = &to {
            executor.transact_call(
                from.unwrap_or_default(),
                *addr,
                U256::default(),
                data,
                u64::MAX,
                Vec::new(),
            )
        } else {
            executor.transact_create(
                from.unwrap_or_default(),
                U256::default(),
                data,
                u64::MAX,
                Vec::new(),
            )
        };

        TxResp {
            exit_reason:  exit,
            ret:          res,
            remain_gas:   executor.gas(),
            gas_used:     executor.used_gas(),
            logs:         vec![],
            code_address: if to.is_none() {
                Some(
                    executor
                        .create_address(CreateScheme::Legacy {
                            caller: from.unwrap_or_default(),
                        })
                        .into(),
                )
            } else {
                None
            },
            removed:      false,
        }
    }

    // Function execute returns exit_reason, ret_data and remain_gas.
    fn exec<B: Backend + ApplyBackend + Adapter>(
        &self,
        backend: &mut B,
        txs: Vec<SignedTransaction>,
    ) -> ExecResp {
        let txs_len = txs.len();
        let mut res = Vec::with_capacity(txs_len);
        let mut hashes = Vec::with_capacity(txs_len);
        let mut gas_use = 0u64;

        let evm_executor = EvmExecutor::new();
        let sys_executor = SystemExecutor::new();

        for tx in txs.into_iter() {
            backend.set_gas_price(tx.transaction.unsigned.gas_price);
            let mut r = if is_call_system_script(&tx.transaction.unsigned.action) {
                sys_executor.inner_exec(backend, tx)
            } else {
                evm_executor.inner_exec(backend, tx)
            };

            r.logs = backend.get_logs();
            gas_use += r.gas_used;

            hashes.push(Hasher::digest(&r.ret));
            res.push(r);
        }
        // commit changes by all txs included in this block only once
        let new_state_root = backend.commit();

        ExecResp {
            state_root:   new_state_root,
            receipt_root: Merkle::from_hashes(hashes)
                .get_root_hash()
                .unwrap_or_default(),
            gas_used:     gas_use,
            tx_resp:      res,
        }
    }

    fn get_account<B: Backend + Adapter>(&self, backend: &B, address: &H160) -> Account {
        match backend.get(address.as_bytes()) {
            Some(bytes) => Account::decode(bytes).unwrap(),
            None => Account {
                nonce:        Default::default(),
                balance:      Default::default(),
                storage_root: RLP_NULL,
                code_hash:    NIL_DATA,
            },
        }
    }
}

pub fn is_call_system_script(action: &TransactionAction) -> bool {
    match action {
        TransactionAction::Call(addr) => addr == &NATIVE_TOKEN_ISSUE_ADDRESS,
        TransactionAction::Create => false,
    }
}
