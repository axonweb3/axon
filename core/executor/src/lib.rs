#![feature(test)]

pub mod adapter;
#[cfg(test)]
mod debugger;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};

use common_merkle::Merkle;
use protocol::codec::ProtocolCodec;
use protocol::traits::{ApplyBackend, Backend, Executor, ExecutorAdapter as Adapter};
use protocol::types::{
    Account, Config, ExecResp, Hasher, SignedTransaction, TransactionAction, TxResp, H160, H256,
    NIL_DATA, RLP_NULL, U256,
};

pub use crate::adapter::{EVMExecutorAdapter, MPTTrie, RocksTrieDB};

#[derive(Default)]
pub struct EvmExecutor;

impl EvmExecutor {
    pub fn new() -> Self {
        EvmExecutor::default()
    }
}

impl Executor for EvmExecutor {
    // Used for query data API, this function will not modify the world state.
    fn call<B: Backend>(&self, backend: &mut B, addr: H160, data: Vec<u8>) -> TxResp {
        let config = Config::london();
        let metadata = StackSubstateMetadata::new(u64::MAX, &config);
        let state = MemoryStackState::new(metadata, backend);
        let precompiles = BTreeMap::new();
        let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);
        let (exit_reason, ret) = executor.transact_call(
            Default::default(),
            addr,
            U256::default(),
            data,
            u64::MAX,
            Vec::new(),
        );

        TxResp {
            exit_reason,
            ret,
            remain_gas: 0,
            gas_used: 0,
            logs: vec![],
            code_address: None,
        }
    }

    // Function execute returns exit_reason, ret_data and remain_gas.
    fn exec<B: Backend + ApplyBackend + Adapter>(
        &self,
        backend: &mut B,
        txs: Vec<SignedTransaction>,
    ) -> ExecResp {
        let mut res = Vec::new();
        let mut hashes = Vec::new();
        let mut gas_use = 0u64;

        txs.into_iter().for_each(|tx| {
            backend.set_gas_price(tx.transaction.unsigned.gas_price);
            let mut r = self.inner_exec(backend, tx);
            r.logs = backend.get_logs();
            gas_use += r.gas_used;

            log::warn!("[exec] resp {:?}", r);
            hashes.push(Hasher::digest(&r.ret));
            res.push(r);
        });

        ExecResp {
            state_root:   backend.state_root(),
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

impl EvmExecutor {
    fn inner_exec<B: Backend + ApplyBackend>(
        &self,
        backend: &mut B,
        tx: SignedTransaction,
    ) -> TxResp {
        let old_nonce = backend.basic(tx.sender).nonce;
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
                tx.transaction.unsigned.data.to_vec(),
                tx.transaction.unsigned.gas_limit.as_u64(),
                tx.transaction
                    .unsigned
                    .access_list
                    .into_iter()
                    .map(|x| (x.address, x.slots))
                    .collect(),
            ),
            TransactionAction::Create => {
                let exit_reason = executor.transact_create(
                    tx.sender,
                    tx.transaction.unsigned.value,
                    tx.transaction.unsigned.data.to_vec(),
                    tx.transaction.unsigned.gas_limit.as_u64(),
                    tx.transaction
                        .unsigned
                        .access_list
                        .into_iter()
                        .map(|x| (x.address, x.slots))
                        .collect(),
                );

                (exit_reason, Vec::new())
            }
        };
        let remain_gas = executor.gas();
        let gas_used = executor.used_gas();

        let code_address = if exit_reason.is_succeed() {
            let (values, logs) = executor.into_state().deconstruct();
            backend.apply(values, logs, true);
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
