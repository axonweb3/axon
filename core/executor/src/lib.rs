pub mod adapter;
#[cfg(test)]
mod debugger;
mod precompiles;
pub mod system_contract;
#[cfg(test)]
mod tests;
mod utils;

pub use crate::adapter::{
    AxonExecutorApplyAdapter, AxonExecutorReadOnlyAdapter, MPTTrie, RocksTrieDB,
};
pub use crate::system_contract::{
    metadata::{MetadataHandle, HARDFORK_INFO},
    DataProvider,
};
pub use crate::utils::{code_address, decode_revert_msg, DefaultFeeAllocator, FeeInlet};

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::iter::FromIterator;

use arc_swap::ArcSwap;
use common_config_parser::types::spec::HardforkName;
use evm::executor::stack::{MemoryStackState, PrecompileFn, StackExecutor, StackSubstateMetadata};
use evm::CreateScheme;

use common_merkle::TrieMerkle;
use protocol::traits::{Backend, Executor, ExecutorAdapter};
use protocol::types::{
    data_gas_cost, logs_bloom, Config, ExecResp, Hasher, SignedTransaction, TransactionAction,
    TxResp, ValidatorExtend, GAS_CALL_TRANSACTION, GAS_CREATE_TRANSACTION, H160, H256, RLP_NULL,
    U256,
};

use crate::precompiles::build_precompile_set;
use crate::system_contract::{
    after_block_hook, before_block_hook, system_contract_dispatch,
    CKB_LIGHT_CLIENT_CONTRACT_ADDRESS, HEADER_CELL_ROOT_KEY, IMAGE_CELL_CONTRACT_ADDRESS,
    METADATA_CONTRACT_ADDRESS, METADATA_ROOT_KEY, NATIVE_TOKEN_CONTRACT_ADDRESS,
};

lazy_static::lazy_static! {
    pub static ref FEE_ALLOCATOR: ArcSwap<Box<dyn FeeAllocate>> = ArcSwap::from_pointee(Box::new(DefaultFeeAllocator));
}

thread_local! {
    pub(crate) static CURRENT_HEADER_CELL_ROOT: RefCell<H256> = RefCell::new(H256::default());
    pub(crate) static CURRENT_METADATA_ROOT: RefCell<H256> = RefCell::new(H256::default());
}

pub trait FeeAllocate: Sync + Send {
    fn allocate(
        &self,
        block_number: U256,
        fee_collect: U256,
        proposer: H160,
        validators: &[ValidatorExtend],
    ) -> Vec<FeeInlet>;
}

#[derive(Default)]
pub struct AxonExecutor;

impl Executor for AxonExecutor {
    // Used for query data API, this function will not modify the global state.
    fn call<B: Backend>(
        &self,
        backend: &B,
        gas_limit: u64,
        from: Option<H160>,
        to: Option<H160>,
        value: U256,
        data: Vec<u8>,
    ) -> TxResp {
        self.init_local_system_contract_roots(backend);
        let config = self.config();
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

        let used_gas = executor.used_gas() + base_gas;

        TxResp {
            exit_reason:  exit,
            ret:          res,
            remain_gas:   executor.gas(),
            gas_used:     used_gas,
            fee_cost:     backend
                .gas_price()
                .checked_mul(used_gas.into())
                .unwrap_or(U256::max_value()),
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
    fn exec<Adapter: ExecutorAdapter>(
        &self,
        adapter: &mut Adapter,
        txs: &[SignedTransaction],
        validators: &[ValidatorExtend],
    ) -> ExecResp {
        let txs_len = txs.len();
        let block_number = adapter.block_number();
        let mut res = Vec::with_capacity(txs_len);
        let mut hashes = Vec::with_capacity(txs_len);
        let (mut gas, mut fee) = (0u64, U256::zero());
        let precompiles = build_precompile_set();
        self.init_local_system_contract_roots(adapter);
        let config = self.config();

        // Execute system contracts before block hook.
        before_block_hook(adapter);

        for tx in txs.iter() {
            adapter.set_gas_price(tx.transaction.unsigned.gas_price());
            adapter.set_origin(tx.sender);

            // Execute a transaction, if system contract dispatch return None, means the
            // transaction called EVM
            let mut r = system_contract_dispatch(adapter, tx)
                .unwrap_or_else(|| Self::evm_exec(adapter, &config, &precompiles, tx));

            r.logs = adapter.take_logs();
            gas += r.gas_used;
            fee = fee.checked_add(r.fee_cost).unwrap_or(U256::max_value());

            let logs_bloom = logs_bloom(r.logs.iter());
            let receipt = tx.encode_receipt(&r, logs_bloom);

            hashes.push(Hasher::digest(&receipt));
            res.push(r);
        }

        // Allocate collected fee for validators
        if !block_number.is_zero() {
            let alloc =
                (*FEE_ALLOCATOR)
                    .load()
                    .allocate(block_number, fee, adapter.origin(), validators);

            for i in alloc.iter() {
                if !i.amount.is_zero() {
                    let mut account = adapter.get_account(&i.address);
                    account.balance += i.amount;
                    adapter.save_account(&i.address, &account);
                }
            }
        }

        // Execute system contracts after block hook.
        after_block_hook(adapter);

        // commit changes by all txs included in this block only once
        let new_state_root = adapter.commit();

        // self.update_system_contract_roots_for_external_module();

        let receipt_root = if hashes.is_empty() {
            RLP_NULL
        } else {
            TrieMerkle::from_iter(hashes.iter().enumerate())
                .root_hash()
                .unwrap_or_else(|err| {
                    panic!("failed to calculate trie root hash for receipts since {err}")
                })
        };

        ExecResp {
            state_root: new_state_root,
            receipt_root,
            gas_used: gas,
            tx_resp: res,
        }
    }
}

impl AxonExecutor {
    pub fn evm_exec<Adapter: ExecutorAdapter>(
        adapter: &mut Adapter,
        config: &Config,
        precompiles: &BTreeMap<H160, PrecompileFn>,
        tx: &SignedTransaction,
    ) -> TxResp {
        // Deduct pre-pay gas
        let sender = tx.sender;
        let tx_gas_price = adapter.gas_price();
        let gas_limit = tx.transaction.unsigned.gas_limit();
        let prepay_gas = tx_gas_price * gas_limit;

        let mut account = adapter.get_account(&sender);
        let old_nonce = account.nonce;

        account.balance = account.balance.saturating_sub(prepay_gas);
        adapter.save_account(&sender, &account);

        let metadata = StackSubstateMetadata::new(gas_limit.as_u64(), config);
        let mut executor = StackExecutor::new_with_precompiles(
            MemoryStackState::new(metadata, adapter),
            config,
            precompiles,
        );

        let access_list = tx
            .transaction
            .unsigned
            .access_list()
            .into_iter()
            .map(|x| (x.address, x.storage_keys))
            .collect::<Vec<_>>();

        let (exit, res) = match tx.transaction.unsigned.action() {
            TransactionAction::Call(addr) => executor.transact_call(
                tx.sender,
                *addr,
                *tx.transaction.unsigned.value(),
                tx.transaction.unsigned.data().to_vec(),
                gas_limit.as_u64(),
                access_list,
            ),
            TransactionAction::Create => executor.transact_create(
                tx.sender,
                *tx.transaction.unsigned.value(),
                tx.transaction.unsigned.data().to_vec(),
                gas_limit.as_u64(),
                access_list,
            ),
        };

        let remained_gas = executor.gas();
        let used_gas = executor.used_gas();

        let code_addr = if tx.transaction.unsigned.action() == &TransactionAction::Create
            && exit.is_succeed()
        {
            Some(code_address(&tx.sender, &old_nonce))
        } else {
            None
        };

        if exit.is_succeed() {
            let (values, logs) = executor.into_state().deconstruct();
            adapter.apply(values, logs, true);
        }

        let mut account = adapter.get_account(&tx.sender);
        account.nonce = old_nonce + U256::one();

        // Add remain gas
        if remained_gas != 0 {
            let remain_gas = U256::from(remained_gas)
                .checked_mul(tx_gas_price)
                .unwrap_or_else(U256::max_value);
            account.balance = account
                .balance
                .checked_add(remain_gas)
                .unwrap_or_else(U256::max_value);
        }

        adapter.save_account(&tx.sender, &account);

        TxResp {
            exit_reason:  exit,
            ret:          res,
            remain_gas:   remained_gas,
            gas_used:     used_gas,
            fee_cost:     tx_gas_price
                .checked_mul(used_gas.into())
                .unwrap_or(U256::max_value()),
            logs:         vec![],
            code_address: code_addr,
            removed:      false,
        }
    }

    /// The `exec()` function is run in `tokio::task::block_in_place()` and all
    /// the read or write operations are in the scope of exec function. The
    /// thread context is not switched during exec function.
    fn init_local_system_contract_roots<Adapter: Backend>(&self, adapter: &Adapter) {
        CURRENT_HEADER_CELL_ROOT.with(|root| {
            *root.borrow_mut() =
                adapter.storage(CKB_LIGHT_CLIENT_CONTRACT_ADDRESS, *HEADER_CELL_ROOT_KEY);
        });

        CURRENT_METADATA_ROOT.with(|root| {
            *root.borrow_mut() = adapter.storage(METADATA_CONTRACT_ADDRESS, *METADATA_ROOT_KEY);
        });
    }

    fn config(&self) -> Config {
        let mut config = Config::london();
        let create_contract_limit = {
            let latest_hardfork_info = &**HARDFORK_INFO.load();
            let enable_contract_limit_flag =
                H256::from_low_u64_be((HardforkName::Andromeda as u64).to_be());
            if latest_hardfork_info & &enable_contract_limit_flag == enable_contract_limit_flag {
                let handle = MetadataHandle::new(CURRENT_METADATA_ROOT.with(|r| *r.borrow()));
                let config = handle.get_consensus_config().unwrap();
                Some(config.max_contract_limit as usize)
            } else {
                Some(0x6000)
            }
        };
        config.create_contract_limit = create_contract_limit;
        config
    }

    #[cfg(test)]
    fn test_exec<Adapter: ExecutorAdapter>(
        &self,
        adapter: &mut Adapter,
        txs: &[SignedTransaction],
        validators: &[ValidatorExtend],
    ) -> ExecResp {
        let txs_len = txs.len();
        let block_number = adapter.block_number();
        let mut res = Vec::with_capacity(txs_len);
        let mut hashes = Vec::with_capacity(txs_len);
        let (mut gas, mut fee) = (0u64, U256::zero());
        let precompiles = build_precompile_set();
        let config = Config::london();

        for tx in txs.iter() {
            adapter.set_gas_price(tx.transaction.unsigned.gas_price());
            adapter.set_origin(tx.sender);

            // Execute a transaction, if system contract dispatch return None, means the
            // transaction called EVM
            let mut r = system_contract_dispatch(adapter, tx)
                .unwrap_or_else(|| Self::evm_exec(adapter, &config, &precompiles, tx));

            r.logs = adapter.take_logs();
            gas += r.gas_used;
            fee = fee.checked_add(r.fee_cost).unwrap_or(U256::max_value());

            let logs_bloom = logs_bloom(r.logs.iter());
            let receipt = tx.encode_receipt(&r, logs_bloom);

            hashes.push(Hasher::digest(&receipt));
            res.push(r);
        }

        // Allocate collected fee for validators
        if !block_number.is_zero() {
            let alloc =
                (*FEE_ALLOCATOR)
                    .load()
                    .allocate(block_number, fee, adapter.origin(), validators);

            for i in alloc.iter() {
                if !i.amount.is_zero() {
                    let mut account = adapter.get_account(&i.address);
                    account.balance += i.amount;
                    adapter.save_account(&i.address, &account);
                }
            }
        }

        // commit changes by all txs included in this block only once
        let new_state_root = adapter.commit();

        let receipt_root = if hashes.is_empty() {
            RLP_NULL
        } else {
            TrieMerkle::from_iter(hashes.iter().enumerate())
                .root_hash()
                .unwrap_or_else(|err| {
                    panic!("failed to calculate trie root hash for receipts since {err}")
                })
        };

        ExecResp {
            state_root: new_state_root,
            receipt_root,
            gas_used: gas,
            tx_resp: res,
        }
    }
}

pub fn is_call_system_script(action: &TransactionAction) -> bool {
    let system_contracts = [
        NATIVE_TOKEN_CONTRACT_ADDRESS,
        METADATA_CONTRACT_ADDRESS,
        CKB_LIGHT_CLIENT_CONTRACT_ADDRESS,
        IMAGE_CELL_CONTRACT_ADDRESS,
    ];

    match action {
        TransactionAction::Call(addr) => system_contracts.contains(addr),
        TransactionAction::Create => false,
    }
}

pub fn is_transaction_call(action: &TransactionAction, addr: &H160) -> bool {
    action == &TransactionAction::Call(*addr)
}
