mod create2;
mod uniswap2;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use evm::tracing::{Event, EventListener};

use protocol::codec::ProtocolCodec;
use protocol::traits::Backend;
use protocol::trie::Trie as _;
use protocol::types::{
    Account, Eip1559Transaction, ExecResp, ExecutorContext, Hash, Hasher, SignedTransaction,
    UnsignedTransaction, UnverifiedTransaction, H160, H256, NIL_DATA, RLP_NULL, U256,
};

use core_db::RocksAdapter;
use core_storage::ImplStorage;

use crate::adapter::{AxonExecutorApplyAdapter, MPTTrie};
use crate::{AxonExecutor, RocksTrieDB};

pub struct EvmDebugger {
    state_root: H256,
    storage:    Arc<ImplStorage<RocksAdapter>>,
    trie_db:    Arc<RocksTrieDB>,
}

impl EvmDebugger {
    pub fn new(distribute_addresses: Vec<H160>, distribute_amount: U256, db_path: &str) -> Self {
        let mut db_data_path = db_path.to_string();
        db_data_path.push_str("/data");
        let _ = std::fs::create_dir_all(&db_data_path);
        let rocks_adapter = Arc::new(RocksAdapter::new(db_data_path, Default::default()).unwrap());

        let mut db_state_path = db_path.to_string();
        db_state_path.push_str("/state");
        let _ = std::fs::create_dir_all(&db_state_path);
        let inner_db = rocks_adapter.inner_db();
        let trie = Arc::new(RocksTrieDB::new_evm(inner_db, 1000));

        let mut mpt = MPTTrie::new(Arc::clone(&trie));

        for distribute_address in distribute_addresses.into_iter() {
            let distribute_account = Account {
                nonce:        U256::zero(),
                balance:      distribute_amount,
                storage_root: RLP_NULL,
                code_hash:    NIL_DATA,
            };

            mpt.insert(
                distribute_address.as_bytes().to_vec(),
                distribute_account.encode().unwrap().to_vec(),
            )
            .unwrap();
        }

        EvmDebugger {
            state_root: mpt.commit().unwrap(),
            storage:    Arc::new(ImplStorage::new(rocks_adapter, 10)),
            trie_db:    trie,
        }
    }

    pub fn exec(&mut self, number: u64, txs: Vec<SignedTransaction>) -> ExecResp {
        let mut backend = self.backend(number);
        let res = AxonExecutor.test_exec(&mut backend, &txs, &[]);
        self.state_root = res.state_root;
        res
    }

    fn backend(
        &self,
        number: u64,
    ) -> AxonExecutorApplyAdapter<ImplStorage<RocksAdapter>, RocksTrieDB> {
        let exec_ctx = ExecutorContext {
            block_number:           number.into(),
            block_coinbase:         H160::random(),
            block_timestamp:        time_now().into(),
            chain_id:               5u64.into(),
            origin:                 H160::random(),
            gas_price:              1u64.into(),
            block_gas_limit:        4294967295000u64.into(),
            block_base_fee_per_gas: 1337u64.into(),
            extra_data:             Default::default(),
        };

        AxonExecutorApplyAdapter::from_root(
            self.state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            exec_ctx,
        )
        .unwrap()
    }

    fn nonce(&self, addr: H160) -> U64 {
        self.backend(0).basic(addr).nonce.low_u64().into()
    }
}

#[derive(Default)]
pub struct EvmListener;

impl EventListener for EvmListener {
    fn event(&mut self, event: Event) {
        println!("EVM event {:?}", event);
    }
}

pub fn mock_signed_tx(tx: Eip1559Transaction, sender: H160) -> SignedTransaction {
    let utx =
        UnverifiedTransaction {
            unsigned:  UnsignedTransaction::Eip1559(tx),
            hash:      Hash::default(),
            chain_id:  Some(5u64),
            signature: None,
        };

    SignedTransaction {
        transaction: utx,
        sender,
        public: None,
    }
}

fn time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
