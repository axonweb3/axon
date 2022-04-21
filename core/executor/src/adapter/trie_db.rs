use std::{collections::HashSet, fs, io, path::Path, sync::Arc};

use dashmap::DashMap;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rocksdb::ops::{Get, Open, Put, WriteOps};
use rocksdb::{FullOptions, Options, WriteBatch, DB};

use common_apm::metrics::storage::{on_storage_get_state, on_storage_put_state};
use common_apm::Instant;
use common_config_parser::types::ConfigRocksDB;
use protocol::{Display, From, ProtocolError, ProtocolErrorKind, ProtocolResult};

// 49999 is the largest prime number within 50000.
const RAND_SEED: u64 = 49999;
const DEFAULT_CACHE_SIZE: usize = 128 << 20;

pub struct RocksTrieDB {
    db:         Arc<DB>,
    cache:      DashMap<Vec<u8>, Vec<u8>>,
    cache_size: usize,
}

impl RocksTrieDB {
    pub fn new<P: AsRef<Path>>(
        path: P,
        config: ConfigRocksDB,
        cache_size: usize,
    ) -> ProtocolResult<Self> {
        if !path.as_ref().is_dir() {
            fs::create_dir_all(&path).map_err(RocksTrieDBError::CreateDB)?;
        }

        let mut opts = if let Some(ref file) = config.options_file {
            let cache_size = match config.cache_size {
                Some(0) => None,
                Some(size) => Some(size),
                None => Some(DEFAULT_CACHE_SIZE),
            };

            let full_opts = FullOptions::load_from_file(file, cache_size, false)
                .map_err(RocksTrieDBError::from)?;

            let FullOptions { db_opts, .. } = full_opts;
            db_opts
        } else {
            Options::default()
        };
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(config.max_open_files);

        let db = DB::open(&opts, path).map_err(RocksTrieDBError::from)?;

        // Init HashMap with capacity 2 * cache_size to avoid reallocate memory.
        Ok(RocksTrieDB {
            db: Arc::new(db),
            cache: DashMap::with_capacity(cache_size + cache_size),
            cache_size,
        })
    }

    fn inner_get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, RocksTrieDBError> {
        use cita_trie::DB;

        let res = self.cache.get(key);

        if res.is_none() {
            let inst = Instant::now();
            let ret = self.db.get(key).map_err(to_store_err)?.map(|r| r.to_vec());
            on_storage_get_state(inst.elapsed(), 1.0);

            if let Some(val) = &ret {
                self.cache.insert(key.to_owned(), val.clone());
                self.flush()?;
            }

            return Ok(ret);
        }

        Ok(Some(res.unwrap().clone()))
    }

    #[cfg(test)]
    fn cache_get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.cache.get(key).map(|v| v.value().to_vec())
    }

    #[cfg(test)]
    fn cache_len(&self) -> usize {
        self.cache.len()
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

        self.db.put(&key, &value).map_err(to_store_err)?;

        {
            self.cache.insert(key, value);
        }

        on_storage_put_state(inst.elapsed(), size as f64);

        self.flush()
    }

    fn insert_batch(&self, keys: Vec<Vec<u8>>, values: Vec<Vec<u8>>) -> Result<(), Self::Error> {
        if keys.len() != values.len() {
            return Err(RocksTrieDBError::BatchLengthMismatch);
        }

        let mut total_size = 0;
        let mut batch = WriteBatch::default();

        {
            for (key, val) in keys.into_iter().zip(values.into_iter()) {
                total_size += key.len();
                total_size += val.len();
                batch.put(&key, &val)?;
                self.cache.insert(key, val);
            }
        }

        let inst = Instant::now();
        self.db.write(&batch).map_err(to_store_err)?;
        on_storage_put_state(inst.elapsed(), total_size as f64);

        self.flush()
    }

    fn remove(&self, _key: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn remove_batch(&self, _keys: &[Vec<u8>]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn flush(&self) -> Result<(), Self::Error> {
        let len = self.cache.len();

        if len <= self.cache_size * 2 {
            return Ok(());
        }

        let keys = self
            .cache
            .iter()
            .map(|kv| kv.key().clone())
            .collect::<Vec<_>>();
        let remove_list = rand_remove_list(keys, len - self.cache_size);

        for item in remove_list {
            self.cache.remove(&item);
        }
        Ok(())
    }
}

fn rand_remove_list<T: Clone>(keys: Vec<T>, num: usize) -> impl Iterator<Item = T> {
    let mut len = keys.len() - 1;
    let mut idx_list = (0..len).collect::<Vec<_>>();
    let mut rng = SmallRng::seed_from_u64(RAND_SEED);
    let mut ret = HashSet::with_capacity(num);

    for _ in 0..num {
        let tmp = rng.gen_range(0..len);
        let idx = idx_list.remove(tmp);
        ret.insert(idx);
        len -= 1;
    }

    keys.into_iter()
        .enumerate()
        .filter_map(move |(i, v)| if ret.contains(&i) { Some(v) } else { None })
}

#[derive(Debug, Display, From)]
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

impl std::error::Error for RocksTrieDBError {}

impl From<RocksTrieDBError> for ProtocolError {
    fn from(err: RocksTrieDBError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Executor, Box::new(err))
    }
}

fn to_store_err(e: rocksdb::Error) -> RocksTrieDBError {
    log::error!("[executor] trie db {:?}", e);
    RocksTrieDBError::Store
}

#[cfg(test)]
mod tests {
    extern crate test;
    use cita_trie::DB;
    use getrandom::getrandom;
    use test::Bencher;

    use super::*;

    fn rand_bytes(len: usize) -> Vec<u8> {
        let mut ret = (0..len).map(|_| 0u8).collect::<Vec<_>>();
        getrandom(&mut ret).unwrap();
        ret
    }

    #[test]
    fn test_rand_remove() {
        let list = (0..10).collect::<Vec<_>>();
        let keys = list.iter().collect::<Vec<_>>();

        for num in 1..10 {
            let res = rand_remove_list(keys.clone(), num);
            assert_eq!(res.count(), num);
        }
    }

    #[test]
    fn test_trie_insert() {
        let key_1 = rand_bytes(32);
        let val_1 = rand_bytes(128);
        let key_2 = rand_bytes(32);
        let val_2 = rand_bytes(256);

        let dir = tempfile::tempdir().unwrap();
        let trie = RocksTrieDB::new(dir.path(), Default::default(), 100).unwrap();

        trie.insert(key_1.clone(), val_1.clone()).unwrap();
        trie.insert(key_2.clone(), val_2.clone()).unwrap();

        let get_1 = trie.get(&key_1).unwrap();
        assert_eq!(val_1, get_1.unwrap());

        let get_2 = trie.get(&key_2).unwrap();
        assert_eq!(val_2, get_2.unwrap());

        let val_3 = rand_bytes(256);
        trie.insert(key_1.clone(), val_3.clone()).unwrap();
        let get_3 = trie.get(&key_1).unwrap();
        assert_eq!(val_3, get_3.unwrap());

        dir.close().unwrap();
    }

    #[test]
    fn test_trie_cache() {
        let key_1 = rand_bytes(32);
        let val_1 = rand_bytes(128);
        let key_2 = rand_bytes(32);
        let val_2 = rand_bytes(256);

        let dir = tempfile::tempdir().unwrap();
        let trie = RocksTrieDB::new(dir.path(), Default::default(), 100).unwrap();

        trie.insert(key_1.clone(), val_1.clone()).unwrap();
        trie.insert(key_2.clone(), val_2.clone()).unwrap();

        let get_1 = trie.get(&key_1).unwrap();
        assert_eq!(val_1, get_1.unwrap());
        assert_eq!(trie.cache_len(), 2);

        let get_2 = trie.get(&key_2).unwrap();
        assert_eq!(val_2, get_2.unwrap());
        assert_eq!(trie.cache_len(), 2);

        let get_1 = trie.cache_get(&key_1).unwrap();
        assert_eq!(val_1, get_1);

        let val_3 = rand_bytes(256);
        trie.insert(key_1.clone(), val_3.clone()).unwrap();
        let get_3 = trie.cache_get(&key_1).unwrap();
        assert_eq!(val_3, get_3);
        assert_eq!(trie.cache_len(), 2);

        let val = rand_bytes(16);
        for _i in 0..198 {
            let key = rand_bytes(128);
            trie.insert(key, val.clone()).unwrap();
        }

        assert_eq!(trie.cache_len(), 200);

        trie.insert(rand_bytes(256), rand_bytes(8)).unwrap();
        assert_eq!(trie.cache_len(), 100);

        dir.close().unwrap();
    }

    #[bench]
    fn bench_rand(b: &mut Bencher) {
        b.iter(|| {
            let mut rng = SmallRng::seed_from_u64(RAND_SEED);
            for _ in 0..10000 {
                rng.gen_range(10..1000000);
            }
        })
    }
}
