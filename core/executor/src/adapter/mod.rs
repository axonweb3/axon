mod trie;
pub mod trie_db;

use std::sync::Arc;

use cita_trie::DB as TrieDB;
use evm::backend::{Apply, Basic};

use protocol::codec::ProtocolCodec;
use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{Account, Bytes, ExecutorContext, Hasher, Log, MerkleRoot, H160, H256, U256};
use protocol::ProtocolResult;

use trie::MPTTrie;

pub struct ExecutorAdapter<DB: TrieDB> {
    trie:     MPTTrie<DB>,
    db:       Arc<DB>,
    exec_ctx: ExecutorContext,
}

impl<DB: TrieDB> Backend for ExecutorAdapter<DB> {
    fn gas_price(&self) -> U256 {
        self.exec_ctx.gas_price
    }

    fn origin(&self) -> H160 {
        self.exec_ctx.origin
    }

    fn block_number(&self) -> U256 {
        self.exec_ctx.block_number
    }

    fn block_hash(&self, _number: U256) -> H256 {
        self.exec_ctx.block_hash
    }

    fn block_coinbase(&self) -> H160 {
        self.exec_ctx.block_coinbase
    }

    fn block_timestamp(&self) -> U256 {
        self.exec_ctx.block_timestamp
    }

    fn block_difficulty(&self) -> U256 {
        self.exec_ctx.difficulty
    }

    fn block_gas_limit(&self) -> U256 {
        self.exec_ctx.block_gas_limit
    }

    fn block_base_fee_per_gas(&self) -> U256 {
        self.exec_ctx.block_base_fee_per_gas
    }

    fn chain_id(&self) -> U256 {
        self.exec_ctx.chain_id
    }

    fn exists(&self, address: H160) -> bool {
        self.trie
            .contains(&Bytes::from(address.as_bytes().to_vec()))
            .unwrap_or_default()
    }

    fn basic(&self, address: H160) -> Basic {
        self.trie
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
        if let Ok(raw) = self.trie.get(address.as_bytes()) {
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

impl<DB: TrieDB> ApplyBackend for ExecutorAdapter<DB> {
    fn apply<A, I, L>(&mut self, values: A, _logs: L, delete_empty: bool)
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
                        self.trie.remove(address.as_bytes()).unwrap();
                        self.trie.commit().unwrap();
                    }
                }
                Apply::Delete { address } => {
                    let _ = self.trie.remove(address.as_bytes());
                }
            }
        }
    }
}

impl<DB: TrieDB> ExecutorAdapter<DB> {
    pub fn new(
        state_root: MerkleRoot,
        db: Arc<DB>,
        exec_ctx: ExecutorContext,
    ) -> ProtocolResult<Self> {
        let trie = MPTTrie::from_root(state_root, Arc::clone(&db))?;

        Ok(ExecutorAdapter { trie, db, exec_ctx })
    }

    fn apply<I: IntoIterator<Item = (H256, H256)>>(
        &mut self,
        address: H160,
        basic: Basic,
        code: Option<Vec<u8>>,
        storage: I,
        reset_storage: bool,
    ) -> bool {
        let old_account = match self.trie.get(address.as_bytes()) {
            Ok(Some(raw)) => Account::decode(raw).unwrap(),
            _ => Account {
                nonce:        Default::default(),
                balance:      Default::default(),
                storage_root: Default::default(),
                code_hash:    Default::default(),
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

        let new_account = Account {
            nonce: basic.nonce,
            balance: basic.balance,
            code_hash: if let Some(c) = code {
                Hasher::digest(c)
            } else {
                old_account.code_hash
            },
            storage_root,
        };

        let raw = new_account.encode().unwrap();
        self.trie.insert(address.as_bytes(), raw.as_ref()).unwrap();
        self.trie.commit().unwrap();

        new_account.balance == U256::zero()
            && new_account.nonce == U256::zero()
            && new_account.code_hash.is_zero()
    }
}
