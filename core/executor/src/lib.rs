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
pub use crate::system::NATIVE_TOKEN_ISSUE_ADDRESS;
pub use crate::vm::{
    code_address, decode_revert_msg, CROSSCHAIN_CONTRACT_ADDRESS, METADATA_CONTRACT_ADDRESS,
    WCKB_CONTRACT_ADDRESS,
};

use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use evm::CreateScheme;

use common_merkle::Merkle;
use evm::executor::stack::PrecompileFn;
use protocol::codec::ProtocolCodec;
use protocol::traits::{ApplyBackend, Backend, Executor, ExecutorAdapter as Adapter};
use protocol::types::{
    data_gas_cost, Account, Config, ExecResp, Hasher, SignedTransaction, TransactionAction, TxResp,
    GAS_CALL_TRANSACTION, GAS_CREATE_TRANSACTION, H160, NIL_DATA, RLP_NULL, U256,
};
use std::collections::BTreeMap;

use crate::{precompiles::build_precompile_set, system::SystemExecutor};

#[derive(Default)]
pub struct AxonExecutor;

impl AxonExecutor {
    fn exec_single<B: Backend + ApplyBackend + Adapter>(
        backend: &mut B,
        config: &Config,
        precompiles: &BTreeMap<H160, PrecompileFn>,
        tx: SignedTransaction,
    ) -> TxResp {
        let old_nonce = backend.basic(tx.sender).nonce;
        let gas_limit = tx.transaction.unsigned.gas_limit();
        let gas_price = tx.transaction.unsigned.gas_price();

        backend.set_gas_price(gas_price);
        backend.set_origin(tx.sender);

        if is_call_system_script(tx.transaction.unsigned.action()) {
            return SystemExecutor::new().inner_exec(backend, tx);
        };

        let mut account = backend.get_account(&tx.sender);
        account.balance = account.balance.saturating_sub(gas_limit * gas_price);
        backend.save_account(&tx.sender, &account);

        let gas_limit = gas_limit.as_u64() - tx.transaction.unsigned.base_gas();
        let gas_limit = gas_limit - data_gas_cost(tx.transaction.unsigned.data());

        let access_list = tx
            .transaction
            .unsigned
            .access_list()
            .into_iter()
            .map(|x| (x.address, x.storage_keys))
            .collect();
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
                access_list,
            ),
            TransactionAction::Create => executor.transact_create(
                tx.sender,
                *tx.transaction.unsigned.value(),
                tx.transaction.unsigned.data().to_vec(),
                gas_limit,
                access_list,
            ),
        };

        let remain_gas = executor.gas();
        let mut account = backend.get_account(&tx.sender);
        account.nonce = old_nonce + U256::one();
        account.balance += U256::from(remain_gas) * gas_price;
        if exit_reason.is_succeed() {
            let (values, logs) = executor.into_state().deconstruct();
            backend.apply(values, logs, true);
        }
        backend.save_account(&tx.sender, &account);

        let code_address = if tx.transaction.unsigned.action() == &TransactionAction::Create
            && exit_reason.is_succeed()
        {
            Some(code_address(&tx.sender, &old_nonce))
        } else {
            None
        };

        TxResp {
            exit_reason,
            ret,
            remain_gas,
            gas_used: tx.transaction.unsigned.gas_limit().as_u64() - remain_gas,
            logs: backend.get_logs(),
            code_address,
            removed: false,
        }
    }
}

impl Executor for AxonExecutor {
    // Used for query data API, this function will not modify the world state.
    fn call<B: Backend>(
        &self,
        backend: &mut B,
        gas_limit: u64,
        from: Option<H160>,
        to: Option<H160>,
        value: U256,
        data: Vec<u8>,
    ) -> TxResp {
        let config = Config::london();
        let metadata = StackSubstateMetadata::new(gas_limit, &config);
        let state = MemoryStackState::new(metadata, backend);
        let precompiles = build_precompile_set();
        let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

        let base_gas = if to.is_some() {
            GAS_CALL_TRANSACTION + data_gas_cost(&data)
        } else {
            GAS_CREATE_TRANSACTION + GAS_CALL_TRANSACTION + data_gas_cost(&data)
        };

        let (exit, res) = if let Some(addr) = &to {
            executor.transact_call(
                from.unwrap_or_default(),
                *addr,
                value,
                data,
                gas_limit,
                Vec::new(),
            )
        } else {
            executor.transact_create(from.unwrap_or_default(), value, data, gas_limit, Vec::new())
        };

        TxResp {
            exit_reason:  exit,
            ret:          res,
            remain_gas:   executor.gas(),
            gas_used:     executor.used_gas() + base_gas,
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

        let precompiles = build_precompile_set();
        let config = Config::london();

        for tx in txs.into_iter() {
            let r = AxonExecutor::exec_single(backend, &config, &precompiles, tx);
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

pub fn is_crosschain_transaction(action: &TransactionAction) -> bool {
    match action {
        TransactionAction::Call(addr) => addr == &CROSSCHAIN_CONTRACT_ADDRESS,
        TransactionAction::Create => false,
    }
}
