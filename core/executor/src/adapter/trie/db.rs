use std::{collections::HashMap, io, sync::Arc};

use parking_lot::RwLock;
use rocksdb::ops::{GetCF, GetColumnFamilys, PutCF, WriteOps};
use rocksdb::{ColumnFamily, WriteBatch, DB};

use common_apm::metrics::storage::{on_storage_get_state, on_storage_put_state};
use common_apm::Instant;
use protocol::rand::{rngs::SmallRng, Rng, SeedableRng};
use protocol::traits::StateStorageCategory;
use protocol::trie;

use core_db::map_category;

// 49999 is the largest prime number within 50000.
const RAND_SEED: u64 = 49999;

macro_rules! db {
    ($db:expr, $op:ident, $column:expr$ (, $args: expr)*) => {
        $db.$op($column, $($args,)*).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("rocksdb error: {:?}", e),
            )
        })?
    };
}

pub struct RocksTrieDB {
    db:         Arc<DB>,
    category:   StateStorageCategory,
    cache:      RwLock<HashMap<Vec<u8>, Vec<u8>>>,
    cache_size: usize,
}

impl trie::DB for RocksTrieDB {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, io::Error> {
        let res = { self.cache.read().get(key).cloned() };

        if res.is_none() {
            let inst = Instant::now();
            let ret = db!(self.db, get_cf, self.get_column(), key);
            on_storage_get_state(inst.elapsed(), 1.0);

            if let Some(val) = &ret {
                {
                    self.cache.write().insert(key.to_owned(), val.to_vec());
                }
                self.flush()?;
            }

            return Ok(ret.map(|r| r.to_vec()));
        }

        Ok(res)
    }

    fn contains(&self, key: &[u8]) -> Result<bool, io::Error> {
        let res = { self.cache.read().contains_key(key) };

        if res {
            Ok(true)
        } else if let Some(val) = db!(self.db, get_cf, self.get_column(), key) {
            self.cache.write().insert(key.to_owned(), val.to_vec());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), io::Error> {
        let inst = Instant::now();
        let size = key.len() + value.len();

        db!(self.db, put_cf, self.get_column(), &key, &value);

        {
            self.cache.write().insert(key, value);
        }

        on_storage_put_state(inst.elapsed(), size as f64);
        self.flush()
    }

    fn insert_batch(&self, keys: Vec<Vec<u8>>, values: Vec<Vec<u8>>) -> Result<(), io::Error> {
        if keys.len() != values.len() {
            return Err(io::Error::new(io::ErrorKind::Other, "keys and values length not match"));
        }

        let mut total_size = 0;
        let mut batch = WriteBatch::default();

        {
            let mut cache = self.cache.write();
            for (key, val) in keys.into_iter().zip(values.into_iter()) {
                total_size += key.len();
                total_size += val.len();

                let column = self.get_column();
                db!(batch, put_cf, column, &key, &val);
                cache.insert(key, val);
            }
        }

        let inst = Instant::now();
        self.db
            .write(&batch)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("rocksdb error: {:?}", e)))?;
        on_storage_put_state(inst.elapsed(), total_size as f64);

        self.flush()
    }

    fn remove(&self, _key: &[u8]) -> Result<(), io::Error> {
        Ok(())
    }

    fn remove_batch(&self, _keys: &[Vec<u8>]) -> Result<(), io::Error> {
        Ok(())
    }

    fn flush(&self) -> Result<(), io::Error> {
        let mut cache = self.cache.write();

        let len = cache.len();

        if len <= self.cache_size * 2 {
            return Ok(());
        }

        let remove_list = {
            let keys = cache.iter().map(|(k, _)| k).collect::<Vec<_>>();
            rand_remove_list(keys, len - self.cache_size)
        };

        for item in remove_list {
            cache.remove(&item);
        }

        Ok(())
    }
}

impl RocksTrieDB {
    pub fn new_evm(db: Arc<DB>, cache_size: usize) -> Self {
        Self::new(db, StateStorageCategory::EvmState, cache_size)
    }

    pub fn new_metadata(db: Arc<DB>, cache_size: usize) -> Self {
        Self::new(db, StateStorageCategory::MetadataState, cache_size)
    }

    pub fn new_ckb_light_client(db: Arc<DB>, cache_size: usize) -> Self {
        Self::new(db, StateStorageCategory::CkbLightClientState, cache_size)
    }

    fn new(db: Arc<DB>, category: StateStorageCategory, cache_size: usize) -> Self {
        let cache = RwLock::new(HashMap::with_capacity(cache_size));
        RocksTrieDB {
            db,
            category,
            cache,
            cache_size,
        }
    }

    fn get_column(&self) -> &ColumnFamily {
        let category = map_category(self.category.into());
        self.db
            .cf_handle(category)
            .unwrap_or_else(|| panic!("Column Family {:?} not found", category))
    }
}

fn rand_remove_list<T: Clone>(keys: Vec<&T>, num: usize) -> impl Iterator<Item = T> {
    let mut len = keys.len() - 1;
    let mut idx_list = (0..len).collect::<Vec<_>>();
    let mut rng = SmallRng::seed_from_u64(RAND_SEED);
    let mut ret = Vec::with_capacity(num);

    for _ in 0..num {
        let tmp = rng.gen_range(0, len);
        let idx = idx_list.remove(tmp);
        ret.push(keys[idx].clone());
        len -= 1;
    }

    ret.into_iter()
}
