mod trie;
mod trie_db;

use std::sync::Arc;

use cita_trie::DB as TrieDB;
use evm::backend::{Apply, Basic};
use parking_lot::Mutex;

use protocol::codec::ProtocolCodec;
use protocol::traits::{ApplyBackend, Backend, ExecutorAdapter};
use protocol::types::{
    Account, Bytes, ExecutorContext, Hasher, Log, MerkleRoot, H160, H256, NIL_DATA, RLP_NULL, U256,
};
use protocol::ProtocolResult;

pub use trie::MPTTrie;
pub use trie_db::RocksTrieDB;

pub struct EVMExecutorAdapter<DB: TrieDB> {
    trie:     Arc<Mutex<MPTTrie<DB>>>,
    db:       Arc<DB>,
    exec_ctx: Arc<Mutex<ExecutorContext>>,
}

impl<DB: TrieDB> ExecutorAdapter for EVMExecutorAdapter<DB> {
    fn get_logs(&self) -> Vec<Log> {
        let mut ret = Vec::new();
        ret.append(&mut self.exec_ctx.lock().logs);
        ret
    }

    fn state_root(&self) -> MerkleRoot {
        self.trie.lock().root
    }

    fn get(&self, key: &[u8]) -> Option<Bytes> {
        self.trie.lock().get(key).ok().flatten()
    }
}

impl<DB: TrieDB> Backend for EVMExecutorAdapter<DB> {
    fn gas_price(&self) -> U256 {
        self.exec_ctx.lock().gas_price
    }

    fn origin(&self) -> H160 {
        self.exec_ctx.lock().origin
    }

    fn block_number(&self) -> U256 {
        self.exec_ctx.lock().block_number
    }

    fn block_hash(&self, _number: U256) -> H256 {
        self.exec_ctx.lock().block_hash
    }

    fn block_coinbase(&self) -> H160 {
        self.exec_ctx.lock().block_coinbase
    }

    fn block_timestamp(&self) -> U256 {
        self.exec_ctx.lock().block_timestamp
    }

    fn block_difficulty(&self) -> U256 {
        self.exec_ctx.lock().difficulty
    }

    fn block_gas_limit(&self) -> U256 {
        self.exec_ctx.lock().block_gas_limit
    }

    fn block_base_fee_per_gas(&self) -> U256 {
        self.exec_ctx.lock().block_base_fee_per_gas
    }

    fn chain_id(&self) -> U256 {
        self.exec_ctx.lock().chain_id
    }

    fn exists(&self, address: H160) -> bool {
        self.trie
            .lock()
            .contains(&Bytes::from(address.as_bytes().to_vec()))
            .unwrap_or_default()
    }

    fn basic(&self, address: H160) -> Basic {
        self.trie
            .lock()
            .get(address.as_bytes())
            .map(|raw| {
                if raw.is_none() {
                    return Basic::default();
                }
                Account::decode(raw.unwrap()).map_or_else(
                    |_| Default::default(),
                    |account| Basic {
                        balance: account.balance,
                        nonce:   account.nonce,
                    },
                )
            })
            .unwrap_or_default()
    }

    fn code(&self, address: H160) -> Vec<u8> {
        self.trie
            .lock()
            .get(address.as_bytes())
            .map(|raw| {
                if raw.is_none() {
                    return Vec::new();
                }
                Account::decode(raw.unwrap()).map_or_else(
                    |_| Default::default(),
                    |account| account.code_hash.as_bytes().to_vec(),
                )
            })
            .unwrap_or_default()
    }

    fn storage(&self, address: H160, index: H256) -> H256 {
        if let Ok(raw) = self.trie.lock().get(address.as_bytes()) {
            if raw.is_none() {
                return H256::default();
            }

            Account::decode(raw.unwrap())
                .and_then(|account| {
                    let storage_root = account.storage_root;
                    MPTTrie::from_root(storage_root, Arc::clone(&self.db)).map(|trie| {
                        match trie.get(index.as_bytes()) {
                            Ok(Some(res)) => H256::from_slice(res.as_ref()),
                            _ => H256::default(),
                        }
                    })
                })
                .unwrap_or_default()
        } else {
            H256::default()
        }
    }

    fn original_storage(&self, address: H160, index: H256) -> Option<H256> {
        Some(self.storage(address, index))
    }
}

impl<DB: TrieDB> ApplyBackend for EVMExecutorAdapter<DB> {
    fn apply<A, I, L>(&mut self, values: A, logs: L, delete_empty: bool)
    where
        A: IntoIterator<Item = Apply<I>>,
        I: IntoIterator<Item = (H256, H256)>,
        L: IntoIterator<Item = Log>,
    {
        for apply in values.into_iter() {
            match apply {
                Apply::Modify {
                    address,
                    basic,
                    code,
                    storage,
                    reset_storage,
                } => {
                    let is_empty = self.apply(address, basic, code, storage, reset_storage);
                    if is_empty && delete_empty {
                        let mut trie = self.trie.lock();
                        trie.remove(address.as_bytes()).unwrap();
                        trie.commit().unwrap();
                    }
                }
                Apply::Delete { address } => {
                    let _ = self.trie.lock().remove(address.as_bytes());
                }
            }
        }

        let logs = logs.into_iter().collect::<Vec<_>>();
        let p = &mut self.exec_ctx.lock().logs;
        p.clear();
        *p = logs;
    }
}

impl<DB: TrieDB> EVMExecutorAdapter<DB> {
    pub fn new(db: Arc<DB>, exec_ctx: Arc<Mutex<ExecutorContext>>) -> ProtocolResult<Self> {
        let trie = Arc::new(Mutex::new(MPTTrie::new(Arc::clone(&db))));
        Ok(EVMExecutorAdapter { trie, db, exec_ctx })
    }

    pub fn from_root(
        state_root: MerkleRoot,
        db: Arc<DB>,
        exec_ctx: Arc<Mutex<ExecutorContext>>,
    ) -> ProtocolResult<Self> {
        let trie = Arc::new(Mutex::new(MPTTrie::from_root(state_root, Arc::clone(&db))?));

        Ok(EVMExecutorAdapter { trie, db, exec_ctx })
    }

    pub fn root(&self) -> MerkleRoot {
        self.trie.lock().root
    }

    fn apply<I: IntoIterator<Item = (H256, H256)>>(
        &mut self,
        address: H160,
        basic: Basic,
        code: Option<Vec<u8>>,
        storage: I,
        reset_storage: bool,
    ) -> bool {
        let old_account = match self.trie.lock().get(address.as_bytes()) {
            Ok(Some(raw)) => Account::decode(raw).unwrap(),
            _ => Account {
                nonce:        Default::default(),
                balance:      Default::default(),
                storage_root: RLP_NULL,
                code_hash:    NIL_DATA,
            },
        };

        let storage_root = if reset_storage {
            H256::default()
        } else {
            let mut storage_trie =
                MPTTrie::from_root(old_account.storage_root, Arc::clone(&self.db)).unwrap();

            storage.into_iter().for_each(|(k, v)| {
                let _ = storage_trie.insert(k.as_bytes(), v.as_bytes());
            });
            storage_trie.commit().unwrap_or_default()
        };
        let new_nonce = basic.nonce.as_u64() + 1;

        let new_account = Account {
            nonce: new_nonce.into(),
            balance: basic.balance,
            code_hash: if let Some(c) = code {
                Hasher::digest(c)
            } else {
                old_account.code_hash
            },
            storage_root,
        };

        let raw = new_account.encode().unwrap();
        {
            let mut trie = self.trie.lock();
            trie.insert(address.as_bytes(), raw.as_ref()).unwrap();
            trie.commit().unwrap();
        }

        new_account.balance == U256::zero()
            && new_account.nonce == U256::zero()
            && new_account.code_hash.is_zero()
    }
}
