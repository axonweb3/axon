#![feature(prelude_import)]
#![feature(test)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub mod adapter {
    mod trie {
        use std::sync::Arc;
        use cita_trie::{PatriciaTrie, Trie, TrieError, DB as TrieDB};
        use hasher::HasherKeccak;
        use protocol::types::{Bytes, Hash, MerkleRoot};
        use protocol::{Display, From, ProtocolError, ProtocolErrorKind, ProtocolResult};
        #[allow(missing_copy_implementations)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        struct HASHER_INST {
            __private_field: (),
        }
        #[doc(hidden)]
        static HASHER_INST: HASHER_INST = HASHER_INST {
            __private_field: (),
        };
        impl ::lazy_static::__Deref for HASHER_INST {
            type Target = Arc<HasherKeccak>;
            fn deref(&self) -> &Arc<HasherKeccak> {
                #[inline(always)]
                fn __static_ref_initialize() -> Arc<HasherKeccak> {
                    Arc::new(HasherKeccak::new())
                }
                #[inline(always)]
                fn __stability() -> &'static Arc<HasherKeccak> {
                    static LAZY: ::lazy_static::lazy::Lazy<Arc<HasherKeccak>> =
                        ::lazy_static::lazy::Lazy::INIT;
                    LAZY.get(__static_ref_initialize)
                }
                __stability()
            }
        }
        impl ::lazy_static::LazyStatic for HASHER_INST {
            fn initialize(lazy: &Self) {
                let _ = &**lazy;
            }
        }
        pub struct MPTTrie<DB: TrieDB> {
            pub root: MerkleRoot,
            trie: PatriciaTrie<DB, HasherKeccak>,
        }
        impl<DB: TrieDB> MPTTrie<DB> {
            pub fn new(db: Arc<DB>) -> Self {
                let trie = PatriciaTrie::new(db, Arc::clone(&HASHER_INST));
                Self {
                    root: Hash::default(),
                    trie,
                }
            }
            pub fn from_root(root: MerkleRoot, db: Arc<DB>) -> ProtocolResult<Self> {
                let trie = PatriciaTrie::from(db, Arc::clone(&HASHER_INST), root.as_bytes())
                    .map_err(MPTTrieError::from)?;
                Ok(Self { root, trie })
            }
            pub fn get(&self, key: &[u8]) -> ProtocolResult<Option<Bytes>> {
                Ok(self
                    .trie
                    .get(key)
                    .map_err(MPTTrieError::from)?
                    .map(Bytes::from))
            }
            pub fn contains(&self, key: &[u8]) -> ProtocolResult<bool> {
                Ok(self.trie.contains(key).map_err(MPTTrieError::from)?)
            }
            pub fn insert(&mut self, key: &[u8], value: &[u8]) -> ProtocolResult<()> {
                self.trie
                    .insert(key.to_vec(), value.to_vec())
                    .map_err(MPTTrieError::from)?;
                Ok(())
            }
            pub fn remove(&mut self, key: &[u8]) -> ProtocolResult<()> {
                if self.trie.remove(key).map_err(MPTTrieError::from)? {
                    Ok(())
                } else {
                    Err(MPTTrieError::RemoveFailed.into())
                }
            }
            pub fn commit(&mut self) -> ProtocolResult<MerkleRoot> {
                let root_bytes = self.trie.root().map_err(MPTTrieError::from)?;
                let root = MerkleRoot::from_slice(&root_bytes);
                self.root = root;
                Ok(root)
            }
        }
        pub enum MPTTrieError {
            #[display(fmt = "{:?}", _0)]
            Trie(TrieError),
            #[display(fmt = "Remove failed")]
            RemoveFailed,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for MPTTrieError {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&MPTTrieError::Trie(ref __self_0),) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "Trie");
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                    (&MPTTrieError::RemoveFailed,) => {
                        ::core::fmt::Formatter::write_str(f, "RemoveFailed")
                    }
                }
            }
        }
        impl ::core::fmt::Display for MPTTrieError {
            #[allow(unused_variables)]
            #[inline]
            fn fmt(
                &self,
                _derive_more_display_formatter: &mut ::core::fmt::Formatter,
            ) -> ::core::fmt::Result {
                match self {
                    MPTTrieError::Trie(_0) => {
                        _derive_more_display_formatter.write_fmt(::core::fmt::Arguments::new_v1(
                            &[""],
                            &match (&_0,) {
                                _args => [::core::fmt::ArgumentV1::new(
                                    _args.0,
                                    ::core::fmt::Debug::fmt,
                                )],
                            },
                        ))
                    }
                    MPTTrieError::RemoveFailed => {
                        _derive_more_display_formatter.write_fmt(::core::fmt::Arguments::new_v1(
                            &["Remove failed"],
                            &match () {
                                _args => [],
                            },
                        ))
                    }
                    _ => Ok(()),
                }
            }
        }
        #[automatically_derived]
        impl ::core::convert::From<()> for MPTTrieError {
            #[inline]
            fn from(original: ()) -> MPTTrieError {
                MPTTrieError::RemoveFailed {}
            }
        }
        #[automatically_derived]
        impl ::core::convert::From<(TrieError)> for MPTTrieError {
            #[inline]
            fn from(original: (TrieError)) -> MPTTrieError {
                MPTTrieError::Trie(original)
            }
        }
        impl std::error::Error for MPTTrieError {}
        impl From<MPTTrieError> for ProtocolError {
            fn from(err: MPTTrieError) -> ProtocolError {
                ProtocolError::new(ProtocolErrorKind::Executor, Box::new(err))
            }
        }
    }
    mod trie_db {
        use std::path::Path;
        use std::sync::Arc;
        use std::{fs, io};
        use dashmap::DashMap;
        use rand::{rngs::SmallRng, Rng, SeedableRng};
        use rocksdb::ops::{Get, Open, Put, WriteOps};
        use rocksdb::{Options, WriteBatch, DB};
        use common_apm::metrics::storage::{on_storage_get_state, on_storage_put_state};
        use common_apm::Instant;
        use protocol::{types::Bytes, Display, From, ProtocolError, ProtocolErrorKind, ProtocolResult};
        const RAND_SEED: u64 = 49999;
        pub struct RocksTrieDB {
            db: Arc<DB>,
            cache: DashMap<Vec<u8>, Vec<u8>>,
            cache_size: usize,
        }
        impl RocksTrieDB {
            pub fn new<P: AsRef<Path>>(
                path: P,
                max_open_files: i32,
                cache_size: usize,
            ) -> ProtocolResult<Self> {
                if !path.as_ref().is_dir() {
                    fs::create_dir_all(&path).map_err(RocksTrieDBError::CreateDB)?;
                }
                let mut opts = Options::default();
                opts.create_if_missing(true);
                opts.create_missing_column_families(true);
                opts.set_max_open_files(max_open_files);
                let db = DB::open(&opts, path).map_err(RocksTrieDBError::from)?;
                Ok(RocksTrieDB {
                    db: Arc::new(db),
                    cache: DashMap::with_capacity(cache_size + cache_size),
                    cache_size,
                })
            }
            fn inner_get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, RocksTrieDBError> {
                let res = self.cache.get(key);
                if res.is_none() {
                    let inst = Instant::now();
                    let ret = self.db.get(key).map_err(to_store_err)?.map(|r| r.to_vec());
                    on_storage_get_state(inst.elapsed(), 1.0);
                    if let Some(val) = &ret {
                        self.cache.insert(key.to_owned(), val.clone());
                    }
                    return Ok(ret);
                }
                Ok(Some(res.unwrap().clone()))
            }
        }
        impl cita_trie::DB for RocksTrieDB {
            type Error = RocksTrieDBError;
            fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
                self.inner_get(key)
            }
            fn contains(&self, key: &[u8]) -> Result<bool, Self::Error> {
                let res = self.cache.contains_key(key);
                if res {
                    Ok(true)
                } else {
                    if let Some(val) = self.db.get(key).map_err(to_store_err)?.map(|r| r.to_vec()) {
                        self.cache.insert(key.to_owned(), val);
                        return Ok(true);
                    }
                    Ok(false)
                }
            }
            fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Self::Error> {
                let inst = Instant::now();
                let size = key.len() + value.len();
                {
                    self.cache.insert(key.clone(), value.clone());
                }
                self.db
                    .put(Bytes::from(key), Bytes::from(value))
                    .map_err(to_store_err)?;
                on_storage_put_state(inst.elapsed(), size as f64);
                Ok(())
            }
            fn insert_batch(
                &self,
                keys: Vec<Vec<u8>>,
                values: Vec<Vec<u8>>,
            ) -> Result<(), Self::Error> {
                if keys.len() != values.len() {
                    return Err(RocksTrieDBError::BatchLengthMismatch);
                }
                let mut total_size = 0;
                let mut batch = WriteBatch::default();
                {
                    for (key, val) in keys.iter().zip(values.iter()) {
                        total_size += key.len();
                        total_size += val.len();
                        batch.put(key, val)?;
                        self.cache.insert(key.clone(), val.clone());
                    }
                }
                let inst = Instant::now();
                self.db.write(&batch).map_err(to_store_err)?;
                on_storage_put_state(inst.elapsed(), total_size as f64);
                Ok(())
            }
            fn remove(&self, _key: &[u8]) -> Result<(), Self::Error> {
                Ok(())
            }
            fn remove_batch(&self, _keys: &[Vec<u8>]) -> Result<(), Self::Error> {
                Ok(())
            }
            fn flush(&self) -> Result<(), Self::Error> {
                let len = self.cache.len();
                if len <= self.cache_size {
                    return Ok(());
                }
                let keys = self
                    .cache
                    .iter()
                    .map(|kv| kv.key().clone())
                    .collect::<Vec<_>>();
                let remove_list = rand_remove_list(keys, len - self.cache_size);
                for item in remove_list.iter() {
                    self.cache.remove(item);
                }
                Ok(())
            }
        }
        fn rand_remove_list<T: Clone>(keys: Vec<T>, num: usize) -> Vec<T> {
            let mut len = keys.len() - 1;
            let mut idx_list = (0..len).collect::<Vec<_>>();
            let mut rng = SmallRng::seed_from_u64(RAND_SEED);
            let mut ret = Vec::with_capacity(num);
            for _ in 0..num {
                let tmp = rng.gen_range(0..len);
                let idx = idx_list.remove(tmp);
                ret.push(keys[idx].to_owned());
                len -= 1;
            }
            ret
        }
        pub enum RocksTrieDBError {
            #[display(fmt = "store error")]
            Store,
            #[display(fmt = "rocksdb {}", _0)]
            RocksDB(rocksdb::Error),
            #[display(fmt = "parameters do not match")]
            InsertParameter,
            #[display(fmt = "batch length do not match")]
            BatchLengthMismatch,
            #[display(fmt = "Create DB path {}", _0)]
            CreateDB(io::Error),
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for RocksTrieDBError {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&RocksTrieDBError::Store,) => ::core::fmt::Formatter::write_str(f, "Store"),
                    (&RocksTrieDBError::RocksDB(ref __self_0),) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "RocksDB");
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                    (&RocksTrieDBError::InsertParameter,) => {
                        ::core::fmt::Formatter::write_str(f, "InsertParameter")
                    }
                    (&RocksTrieDBError::BatchLengthMismatch,) => {
                        ::core::fmt::Formatter::write_str(f, "BatchLengthMismatch")
                    }
                    (&RocksTrieDBError::CreateDB(ref __self_0),) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "CreateDB");
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                }
            }
        }
        impl ::core::fmt::Display for RocksTrieDBError {
            #[allow(unused_variables)]
            #[inline]
            fn fmt(
                &self,
                _derive_more_display_formatter: &mut ::core::fmt::Formatter,
            ) -> ::core::fmt::Result {
                match self {
                    RocksTrieDBError::Store => {
                        _derive_more_display_formatter.write_fmt(::core::fmt::Arguments::new_v1(
                            &["store error"],
                            &match () {
                                _args => [],
                            },
                        ))
                    }
                    RocksTrieDBError::RocksDB(_0) => {
                        _derive_more_display_formatter.write_fmt(::core::fmt::Arguments::new_v1(
                            &["rocksdb "],
                            &match (&_0,) {
                                _args => [::core::fmt::ArgumentV1::new(
                                    _args.0,
                                    ::core::fmt::Display::fmt,
                                )],
                            },
                        ))
                    }
                    RocksTrieDBError::InsertParameter => {
                        _derive_more_display_formatter.write_fmt(::core::fmt::Arguments::new_v1(
                            &["parameters do not match"],
                            &match () {
                                _args => [],
                            },
                        ))
                    }
                    RocksTrieDBError::BatchLengthMismatch => _derive_more_display_formatter
                        .write_fmt(::core::fmt::Arguments::new_v1(
                            &["batch length do not match"],
                            &match () {
                                _args => [],
                            },
                        )),
                    RocksTrieDBError::CreateDB(_0) => {
                        _derive_more_display_formatter.write_fmt(::core::fmt::Arguments::new_v1(
                            &["Create DB path "],
                            &match (&_0,) {
                                _args => [::core::fmt::ArgumentV1::new(
                                    _args.0,
                                    ::core::fmt::Display::fmt,
                                )],
                            },
                        ))
                    }
                    _ => Ok(()),
                }
            }
        }
        #[automatically_derived]
        impl ::core::convert::From<(io::Error)> for RocksTrieDBError {
            #[inline]
            fn from(original: (io::Error)) -> RocksTrieDBError {
                RocksTrieDBError::CreateDB(original)
            }
        }
        #[automatically_derived]
        impl ::core::convert::From<(rocksdb::Error)> for RocksTrieDBError {
            #[inline]
            fn from(original: (rocksdb::Error)) -> RocksTrieDBError {
                RocksTrieDBError::RocksDB(original)
            }
        }
        impl std::error::Error for RocksTrieDBError {}
        impl From<RocksTrieDBError> for ProtocolError {
            fn from(err: RocksTrieDBError) -> ProtocolError {
                ProtocolError::new(ProtocolErrorKind::Executor, Box::new(err))
            }
        }
        fn to_store_err(e: rocksdb::Error) -> RocksTrieDBError {
            {
                let lvl = ::log::Level::Error;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api_log(
                        ::core::fmt::Arguments::new_v1(
                            &["[executor] trie db "],
                            &match (&e,) {
                                _args => [::core::fmt::ArgumentV1::new(
                                    _args.0,
                                    ::core::fmt::Debug::fmt,
                                )],
                            },
                        ),
                        lvl,
                        &(
                            "core_executor::adapter::trie_db",
                            "core_executor::adapter::trie_db",
                            "core/executor/src/adapter/trie_db.rs",
                            209u32,
                        ),
                    );
                }
            };
            RocksTrieDBError::Store
        }
    }
    pub use trie::MPTTrie;
    pub use trie_db::RocksTrieDB;
    use std::sync::Arc;
    use evm::backend::{Apply, Basic};
    use protocol::traits::{ApplyBackend, Backend, Context, ExecutorAdapter, Storage};
    use protocol::types::{
        Account, Bytes, ExecutorContext, Hasher, Log, MerkleRoot, H160, H256, NIL_DATA, RLP_NULL,
        U256,
    };
    use protocol::{codec::ProtocolCodec, ProtocolResult};
    pub struct AxonExecutorAdapter<S, DB: cita_trie::DB> {
        exec_ctx: ExecutorContext,
        trie: MPTTrie<DB>,
        storage: Arc<S>,
        db: Arc<DB>,
    }
    impl<S, DB> ExecutorAdapter for AxonExecutorAdapter<S, DB>
    where
        S: Storage + 'static,
        DB: cita_trie::DB + 'static,
    {
        fn get_ctx(&self) -> ExecutorContext {
            self.exec_ctx.clone()
        }
        fn set_gas_price(&mut self, gas_price: U256) {
            self.exec_ctx.gas_price = gas_price;
        }
        fn get_logs(&mut self) -> Vec<Log> {
            let mut ret = Vec::new();
            ret.append(&mut self.exec_ctx.logs);
            ret
        }
        fn state_root(&self) -> MerkleRoot {
            self.trie.root
        }
        fn get(&self, key: &[u8]) -> Option<Bytes> {
            self.trie.get(key).ok().flatten()
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
                            nonce: account.nonce,
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
            let res = {
                let rt = protocol::tokio::runtime::Handle::current();
                let adapter = Arc::clone(&self.storage);
                protocol::tokio::task::block_in_place(move || {
                    rt.block_on(adapter.get_code_by_hash(Context::new(), &code_hash))
                        .unwrap()
                })
            };
            res.unwrap().to_vec()
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
                            MPTTrie::from_root(storage_root, Arc::clone(&self.db)).map(|trie| {
                                match trie.get(index.as_bytes()) {
                                    Ok(Some(res)) => H256::from_slice(res.as_ref()),
                                    _ => H256::default(),
                                }
                            })
                        }
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
                            self.trie.commit().unwrap();
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
        pub fn new(
            db: Arc<DB>,
            storage: Arc<S>,
            exec_ctx: ExecutorContext,
        ) -> ProtocolResult<Self> {
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
        pub fn root(&self) -> MerkleRoot {
            self.trie.root
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
                    nonce: Default::default(),
                    balance: Default::default(),
                    storage_root: RLP_NULL,
                    code_hash: NIL_DATA,
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
            let new_storage_root = storage_trie.commit().unwrap_or(RLP_NULL);
            let mut new_account = Account {
                nonce: basic.nonce,
                balance: basic.balance,
                code_hash: old_account.code_hash,
                storage_root: new_storage_root,
            };
            if let Some(c) = code {
                let new_code_hash = Hasher::digest(&c);
                if new_code_hash != old_account.code_hash {
                    let _ = {
                        let rt = protocol::tokio::runtime::Handle::current();
                        let adapter = Arc::clone(&self.storage);
                        protocol::tokio::task::block_in_place(move || {
                            rt.block_on(adapter.insert_code(
                                Context::new(),
                                address.into(),
                                new_code_hash,
                                c.into(),
                            ))
                            .unwrap()
                        })
                    };
                    new_account.code_hash = new_code_hash;
                }
            }
            let bytes = new_account.encode().unwrap();
            {
                self.trie
                    .insert(address.as_bytes(), bytes.as_ref())
                    .unwrap();
                self.trie.commit().unwrap();
            }
            new_account.balance == U256::zero()
                && new_account.nonce == U256::zero()
                && new_account.code_hash.is_zero()
        }
    }
}
mod precompiles {
    mod blake2_f {}
    mod ecrecover {}
    mod modexp {}
    mod rsa {}
    mod secp256r1 {}
    mod sha256 {
        use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
        use evm::{Context, ExitError, ExitSucceed};
        use protocol::types::H160;
        use sha2::Digest;
        use crate::precompiles::{precompile_address, PrecompileContract};
        pub struct Sha256;
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for Sha256 {
            #[inline]
            fn default() -> Sha256 {
                Sha256 {}
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Sha256 {
            #[inline]
            fn clone(&self) -> Sha256 {
                match *self {
                    Sha256 => Sha256,
                }
            }
        }
        impl PrecompileContract for Sha256 {
            const ADDRESS: H160 = precompile_address(0x02);
            fn exec_fn(
                input: &[u8],
                gas_limit: Option<u64>,
                _context: &Context,
                is_static: bool,
            ) -> Result<PrecompileOutput, PrecompileFailure> {
                let gas = if is_static {
                    60u64
                } else {
                    let data_word_size = (input.len() + 31) / 32;
                    (data_word_size * 12) as u64
                };
                if let Some(limit) = gas_limit {
                    if gas > limit {
                        return Err(PrecompileFailure::Error {
                            exit_status: ExitError::OutOfGas,
                        });
                    }
                }
                let mut hasher = sha2::Sha256::default();
                hasher.update(input);
                Ok(PrecompileOutput {
                    exit_status: ExitSucceed::Returned,
                    cost: gas,
                    output: hasher.finalize().to_vec(),
                    logs: ::alloc::vec::Vec::new(),
                })
            }
        }
    }
    use std::collections::BTreeMap;
    use evm::executor::stack::{PrecompileFailure, PrecompileFn, PrecompileOutput};
    use evm::Context;
    use protocol::types::H160;
    use crate::precompiles::sha256::Sha256;
    trait PrecompileContract {
        const ADDRESS: H160;
        fn exec_fn(
            input: &[u8],
            gas_limit: Option<u64>,
            context: &Context,
            is_static: bool,
        ) -> Result<PrecompileOutput, PrecompileFailure>;
    }
    const fn precompile_address(addr: u8) -> H160 {
        H160([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, addr,
        ])
    }
    pub fn build_precompile_set() -> BTreeMap<H160, PrecompileFn> {
        {
            let mut set = BTreeMap::new();
            set.insert(Sha256::ADDRESS, Sha256::exec_fn);
            set
        }
    }
}
mod system {
    use protocol::traits::{ApplyBackend, Backend};
    use protocol::types::{
        ExitReason, ExitRevert, SignedTransaction, TransactionAction, TxResp, H160, U256,
    };
    pub const NATIVE_TOKEN_ISSUE_ADDRESS: H160 = H160([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff,
    ]);
    pub struct SystemExecutor;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::default::Default for SystemExecutor {
        #[inline]
        fn default() -> SystemExecutor {
            SystemExecutor {}
        }
    }
    impl SystemExecutor {
        pub fn new() -> Self {
            SystemExecutor::default()
        }
        pub fn inner_exec<B: Backend + ApplyBackend>(
            &self,
            backend: &mut B,
            tx: SignedTransaction,
        ) -> TxResp {
            match classify_script(&tx.transaction.unsigned.action) {
                SystemScriptCategory::NativeToken => native_token::call_native_token(backend, tx),
            }
        }
    }
    enum SystemScriptCategory {
        NativeToken,
    }
    fn classify_script(action: &TransactionAction) -> SystemScriptCategory {
        match action {
            TransactionAction::Call(_addr) => SystemScriptCategory::NativeToken,
            TransactionAction::Create => {
                ::core::panicking::panic("internal error: entered unreachable code")
            }
        }
    }
    fn revert_resp(gas_limit: U256) -> TxResp {
        TxResp {
            exit_reason: ExitReason::Revert(ExitRevert::Reverted),
            ret: ::alloc::vec::Vec::new(),
            gas_used: 1u64,
            remain_gas: (gas_limit - 1).as_u64(),
            logs: ::alloc::vec::Vec::new(),
            code_address: None,
        }
    }
    mod native_token {
        use protocol::codec::ProtocolCodec;
        use protocol::traits::{ApplyBackend, Backend};
        use protocol::types::{
            Apply, Basic, ExitReason, ExitSucceed, SignedTransaction, TxResp, H160, U256,
        };
        use crate::system::revert_resp;
        pub fn call_native_token<B: Backend + ApplyBackend>(
            backend: &mut B,
            tx: SignedTransaction,
        ) -> TxResp {
            let tx = tx.transaction.unsigned;
            if tx.data.len() < 21 || tx.data[0] > 1 {
                return revert_resp(tx.gas_limit);
            }
            let direction = tx.data[0] == 0u8;
            let l2_addr = H160::from_slice(&tx.data[1..21]);
            let mut account = backend.basic(l2_addr);
            if direction {
                account.balance += tx.value;
            } else {
                if account.balance < tx.value {
                    return revert_resp(tx.gas_limit);
                }
                account.balance -= tx.value;
            }
            backend.apply(
                <[_]>::into_vec(box [Apply::Modify {
                    address: l2_addr,
                    basic: Basic {
                        balance: account.balance,
                        nonce: account.nonce + U256::one(),
                    },
                    code: None,
                    storage: ::alloc::vec::Vec::new(),
                    reset_storage: false,
                }]),
                ::alloc::vec::Vec::new(),
                false,
            );
            TxResp {
                exit_reason: ExitReason::Succeed(ExitSucceed::Returned),
                ret: account.balance.encode().unwrap().to_vec(),
                gas_used: 0u64,
                remain_gas: tx.gas_limit.as_u64(),
                logs: ::alloc::vec::Vec::new(),
                code_address: None,
            }
        }
    }
}
mod vm {
    use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
    use protocol::traits::{ApplyBackend, Backend};
    use protocol::types::{
        Config, Hasher, SignedTransaction, TransactionAction, TxResp, H160, H256, U256,
    };
    use crate::precompiles::build_precompile_set;
    pub struct EvmExecutor;
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::default::Default for EvmExecutor {
        #[inline]
        fn default() -> EvmExecutor {
            EvmExecutor {}
        }
    }
    impl EvmExecutor {
        pub fn new() -> Self {
            EvmExecutor::default()
        }
        pub fn inner_exec<B: Backend + ApplyBackend>(
            &self,
            backend: &mut B,
            tx: SignedTransaction,
        ) -> TxResp {
            let old_nonce = backend.basic(tx.sender).nonce;
            let config = Config::london();
            let metadata = StackSubstateMetadata::new(u64::MAX, &config);
            let state = MemoryStackState::new(metadata, backend);
            let precompiles = build_precompile_set();
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
                logs: ::alloc::vec::Vec::new(),
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
}
pub use crate::adapter::{AxonExecutorAdapter, MPTTrie, RocksTrieDB};
pub use crate::{system::NATIVE_TOKEN_ISSUE_ADDRESS, vm::code_address};
use std::collections::BTreeMap;
use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use common_merkle::Merkle;
use protocol::codec::ProtocolCodec;
use protocol::traits::{ApplyBackend, Backend, Executor, ExecutorAdapter as Adapter};
use protocol::types::{
    Account, Config, ExecResp, Hasher, SignedTransaction, TransactionAction, TxResp, H160,
    NIL_DATA, RLP_NULL, U256,
};
use crate::{system::SystemExecutor, vm::EvmExecutor};
pub struct AxonExecutor;
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::default::Default for AxonExecutor {
    #[inline]
    fn default() -> AxonExecutor {
        AxonExecutor {}
    }
}
impl AxonExecutor {
    pub fn new() -> Self {
        AxonExecutor::default()
    }
}
impl Executor for AxonExecutor {
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
            logs: ::alloc::vec::Vec::new(),
            code_address: None,
        }
    }
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
        ExecResp {
            state_root: backend.state_root(),
            receipt_root: Merkle::from_hashes(hashes)
                .get_root_hash()
                .unwrap_or_default(),
            gas_used: gas_use,
            tx_resp: res,
        }
    }
    fn get_account<B: Backend + Adapter>(&self, backend: &B, address: &H160) -> Account {
        match backend.get(address.as_bytes()) {
            Some(bytes) => Account::decode(bytes).unwrap(),
            None => Account {
                nonce: Default::default(),
                balance: Default::default(),
                storage_root: RLP_NULL,
                code_hash: NIL_DATA,
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
