#![allow(dead_code)]

mod uniswap2;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use getrandom::getrandom;

use core_storage::{adapter::rocks::RocksAdapter, ImplStorage};
use protocol::codec::ProtocolCodec;
use protocol::traits::Executor;
use protocol::types::{
    Account, ExecResp, ExecutorContext, Hash, Hasher, SignedTransaction, Transaction,
    UnverifiedTransaction, H160, H256, NIL_DATA, RLP_NULL, U256,
};

use crate::adapter::{EVMExecutorAdapter, MPTTrie};
use crate::{EvmExecutor, RocksTrieDB};

pub struct EvmDebugger {
    state_root: H256,
    storage:    Arc<ImplStorage<RocksAdapter>>,
    trie_db:    Arc<RocksTrieDB>,
}

impl EvmDebugger {
    pub fn new(distribute_address: H160, distribute_amount: U256, db_path: &str) -> Self {
        let mut db_data_path = db_path.to_string();
        db_data_path.push_str("/data");
        let rocks_adapter = Arc::new(RocksAdapter::new(db_data_path, 1024).unwrap());
        let mut db_state_path = db_path.to_string();
        db_state_path.push_str("/state");
        let trie = Arc::new(RocksTrieDB::new(db_state_path, 1024, 1000).unwrap());

        let mut mpt = MPTTrie::new(Arc::clone(&trie));

        let distribute_account = Account {
            nonce:        0u64.into(),
            balance:      distribute_amount,
            storage_root: RLP_NULL,
            code_hash:    NIL_DATA,
        };

        mpt.insert(
            distribute_address.as_bytes(),
            distribute_account.encode().unwrap().as_ref(),
        )
        .unwrap();

        EvmDebugger {
            state_root: mpt.commit().unwrap(),
            storage:    Arc::new(ImplStorage::new(rocks_adapter)),
            trie_db:    trie,
        }
    }

    pub fn exec(&mut self, number: u64, txs: Vec<SignedTransaction>) -> ExecResp {
        let mut backend = self.backend(number);
        let evm = EvmExecutor::default();
        let res = evm.exec(&mut backend, txs);
        self.state_root = res.state_root;
        res
    }

    fn backend(&self, number: u64) -> EVMExecutorAdapter<ImplStorage<RocksAdapter>, RocksTrieDB> {
        let exec_ctx = ExecutorContext {
            block_number:           number.into(),
            block_hash:             rand_hash(),
            block_coinbase:         rand_hash().into(),
            block_timestamp:        time_now().into(),
            chain_id:               5u64.into(),
            difficulty:             U256::one(),
            origin:                 rand_hash().into(),
            gas_price:              1u64.into(),
            block_gas_limit:        4294967295000u64.into(),
            block_base_fee_per_gas: 1337u64.into(),
            logs:                   vec![],
        };

        EVMExecutorAdapter::from_root(
            self.state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            exec_ctx,
        )
        .unwrap()
    }
}

pub fn mock_signed_tx(tx: Transaction, sender: H160) -> SignedTransaction {
    let utx = UnverifiedTransaction {
        unsigned:  tx,
        hash:      Hash::default(),
        chain_id:  5u64,
        signature: None,
    };

    SignedTransaction {
        transaction: utx,
        sender,
        public: None,
    }
}

pub fn clear_data(db_path: &str) {
    std::fs::remove_dir_all(db_path).unwrap()
}

fn rand_hash() -> Hash {
    let mut data = [0u8; 128];
    getrandom(&mut data).unwrap();
    Hasher::digest(&data)
}

fn time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u64
}
