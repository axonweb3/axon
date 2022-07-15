mod trie;
mod trie_db;

pub use trie::MPTTrie;
pub use trie_db::RocksTrieDB;

use std::sync::Arc;

use evm::backend::{Apply, Basic};

use protocol::traits::{ApplyBackend, Backend, Context, ExecutorAdapter, Storage};
use protocol::types::{
    Account, Bytes, ExecutorContext, Hasher, Log, MerkleRoot, H160, H256, NIL_DATA, RLP_NULL, U256,
};
use protocol::{codec::ProtocolCodec, ProtocolResult};

macro_rules! blocking_async {
    ($self_: ident, $adapter: ident, $method: ident$ (, $args: expr)*) => {{
        let rt = protocol::tokio::runtime::Handle::current();
        let adapter = Arc::clone(&$self_.$adapter);

        protocol::tokio::task::block_in_place(move || {
            rt.block_on(adapter.$method( $($args,)* )).unwrap()
        })
    }};
}

pub struct AxonExecutorAdapter<S, DB: cita_trie::DB> {
    exec_ctx: ExecutorContext,
    trie:     MPTTrie<DB>,
    storage:  Arc<S>,
    db:       Arc<DB>,
}

impl<S, DB> ExecutorAdapter for AxonExecutorAdapter<S, DB>
where
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    fn get_ctx(&self) -> ExecutorContext {
        self.exec_ctx.clone()
    }

    fn set_origin(&mut self, origin: H160) {
        self.exec_ctx.origin = origin;
    }

    fn set_gas_price(&mut self, gas_price: U256) {
        self.exec_ctx.gas_price = gas_price;
    }

    fn get_logs(&mut self) -> Vec<Log> {
        let mut ret = Vec::new();
        ret.append(&mut self.exec_ctx.logs);
        ret
    }

    fn commit(&mut self) -> MerkleRoot {
        self.trie.commit().unwrap()
    }

    fn get(&self, key: &[u8]) -> Option<Bytes> {
        self.trie.get(key).ok().flatten()
    }

    fn get_account(&self, address: &H160) -> Account {
        if let Ok(Some(raw)) = self.trie.get(address.as_bytes()) {
            return Account::decode(raw).unwrap();
        }

        Account {
            nonce:        U256::zero(),
            balance:      U256::zero(),
            storage_root: RLP_NULL,
            code_hash:    NIL_DATA,
        }
    }

    fn save_account(&mut self, address: &H160, account: &Account) {
        self.trie
            .insert(address.as_bytes(), &account.encode().unwrap())
            .unwrap();
    }
}

impl<S, DB> Backend for AxonExecutorAdapter<S, DB>
where
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
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
        let code_hash = if let Some(bytes) = self.trie.get(address.as_bytes()).unwrap() {
            Account::decode(bytes).unwrap().code_hash
        } else {
            return Vec::new();
        };

        if code_hash == NIL_DATA {
            return Vec::new();
        }

        let res = blocking_async!(self, storage, get_code_by_hash, Context::new(), &code_hash);

        res.unwrap_or_default().to_vec()
    }

    fn storage(&self, address: H160, index: H256) -> H256 {
        if let Ok(raw) = self.trie.get(address.as_bytes()) {
            if raw.is_none() {
                return H256::default();
            }

            Account::decode(raw.unwrap())
                .and_then(|account| {
                    let storage_root = account.storage_root;
                    if storage_root == RLP_NULL {
                        Ok(H256::default())
                    } else {
                        MPTTrie::from_root(storage_root, Arc::clone(&self.db)).map(
                            |trie| match trie.get(index.as_bytes()) {
                                Ok(Some(res)) => H256::from_slice(res.as_ref()),
                                _ => H256::default(),
                            },
                        )
                    }
                })
                .unwrap_or_default()
        } else {
            H256::default()
        }
    }

    fn original_storage(&self, address: H160, index: H256) -> Option<H256> {
        // Fixme
        Some(self.storage(address, index))
    }
}

impl<S, DB> ApplyBackend for AxonExecutorAdapter<S, DB>
where
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
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
                        self.trie.remove(address.as_bytes()).unwrap();
                    }
                }
                Apply::Delete { address } => {
                    let _ = self.trie.remove(address.as_bytes());
                }
            }
        }

        self.exec_ctx.logs = logs.into_iter().collect::<Vec<_>>();
    }
}

impl<S, DB> AxonExecutorAdapter<S, DB>
where
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    pub fn new(db: Arc<DB>, storage: Arc<S>, exec_ctx: ExecutorContext) -> ProtocolResult<Self> {
        let trie = MPTTrie::new(Arc::clone(&db));
        Ok(AxonExecutorAdapter {
            trie,
            db,
            storage,
            exec_ctx,
        })
    }

    pub fn from_root(
        state_root: MerkleRoot,
        db: Arc<DB>,
        storage: Arc<S>,
        exec_ctx: ExecutorContext,
    ) -> ProtocolResult<Self> {
        let trie = MPTTrie::from_root(state_root, Arc::clone(&db))?;

        Ok(AxonExecutorAdapter {
            trie,
            db,
            storage,
            exec_ctx,
        })
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
                nonce:        U256::zero(),
                balance:      U256::zero(),
                storage_root: RLP_NULL,
                code_hash:    NIL_DATA,
            },
        };

        let storage_root = if reset_storage {
            RLP_NULL
        } else {
            old_account.storage_root
        };

        let mut storage_trie = if storage_root == RLP_NULL {
            MPTTrie::new(Arc::clone(&self.db))
        } else {
            MPTTrie::from_root(old_account.storage_root, Arc::clone(&self.db)).unwrap()
        };

        storage.into_iter().for_each(|(k, v)| {
            let _ = storage_trie.insert(k.as_bytes(), v.as_bytes());
        });

        let mut new_account = Account {
            nonce:        basic.nonce,
            balance:      basic.balance,
            code_hash:    old_account.code_hash,
            storage_root: storage_trie.commit().unwrap_or(RLP_NULL),
        };

        if let Some(c) = code {
            let new_code_hash = Hasher::digest(&c);
            if new_code_hash != old_account.code_hash {
                let _ = blocking_async!(
                    self,
                    storage,
                    insert_code,
                    Context::new(),
                    address.into(),
                    new_code_hash,
                    c.into()
                );

                new_account.code_hash = new_code_hash;
            }
        }

        let bytes = new_account.encode().unwrap();

        {
            self.trie
                .insert(address.as_bytes(), bytes.as_ref())
                .unwrap();
        }

        new_account.balance == U256::zero()
            && new_account.nonce == U256::zero()
            && new_account.code_hash.is_zero()
    }
}
