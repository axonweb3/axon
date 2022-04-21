use std::error::Error;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;
use std::{fs, io};

use rocksdb::ops::{DeleteCF, GetCF, GetColumnFamilys, IterateCF, OpenCF, PutCF, WriteOps};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBIterator, FullOptions, Options, WriteBatch, DB,
};

use common_apm::metrics::storage::on_storage_put_cf;
use common_apm::Instant;
use common_config_parser::types::ConfigRocksDB;
use protocol::codec::{hex_encode, ProtocolCodec};
use protocol::traits::{
    IntoIteratorByRef, StorageAdapter, StorageBatchModify, StorageCategory, StorageIterator,
    StorageSchema,
};
use protocol::{
    async_trait, types::Bytes, Display, From, ProtocolError, ProtocolErrorKind, ProtocolResult,
};

const DEFAULT_CACHE_SIZE: usize = 128 << 20;

#[derive(Debug)]
pub struct RocksAdapter {
    db: Arc<DB>,
}

impl RocksAdapter {
    pub fn new<P: AsRef<Path>>(path: P, config: ConfigRocksDB) -> ProtocolResult<Self> {
        if !path.as_ref().is_dir() {
            fs::create_dir_all(&path).map_err(RocksAdapterError::CreateDB)?;
        }
        let categories = [
            map_category(StorageCategory::Block),
            map_category(StorageCategory::BlockHeader),
            map_category(StorageCategory::Receipt),
            map_category(StorageCategory::SignedTransaction),
            map_category(StorageCategory::Wal),
            map_category(StorageCategory::HashHeight),
            map_category(StorageCategory::Code),
        ];

        let (mut opts, cf_descriptors) = if let Some(ref file) = config.options_file {
            let cache_size = match config.cache_size {
                Some(0) => None,
                Some(size) => Some(size),
                None => Some(DEFAULT_CACHE_SIZE),
            };

            let mut full_opts = FullOptions::load_from_file(file, cache_size, false)
                .map_err(RocksAdapterError::from)?;

            full_opts
                .complete_column_families(&categories, false)
                .map_err(RocksAdapterError::from)?;
            let FullOptions {
                db_opts,
                cf_descriptors,
            } = full_opts;
            (db_opts, cf_descriptors)
        } else {
            let opts = Options::default();
            let cf_descriptors: Vec<_> = categories
                .into_iter()
                .map(|c| ColumnFamilyDescriptor::new(c, Options::default()))
                .collect();
            (opts, cf_descriptors)
        };

        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(config.max_open_files);

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)
            .map_err(RocksAdapterError::from)?;

        Ok(RocksAdapter { db: Arc::new(db) })
    }
}

macro_rules! db {
    ($db:expr, $op:ident, $column:expr, $key:expr) => {
        $db.$op($column, $key).map_err(RocksAdapterError::from)
    };
    ($db:expr, $op:ident, $column:expr, $key:expr, $val:expr) => {
        $db.$op($column, $key, $val)
            .map_err(RocksAdapterError::from)
    };
}

pub struct RocksIterator<'a, S: StorageSchema> {
    inner: DBIterator<'a>,
    pin_s: PhantomData<S>,
}

impl<'a, S: StorageSchema> Iterator for RocksIterator<'a, S> {
    type Item = ProtocolResult<(<S as StorageSchema>::Key, <S as StorageSchema>::Value)>;

    fn next(&mut self) -> Option<Self::Item> {
        let kv_decode = |(k_bytes, v_bytes): (Box<[u8]>, Box<[u8]>)| -> ProtocolResult<_> {
            let k_bytes = Bytes::copy_from_slice(k_bytes.as_ref());
            let key = <_>::decode(k_bytes)?;

            let v_bytes = Bytes::copy_from_slice(v_bytes.as_ref());
            let val = <_>::decode(v_bytes)?;

            Ok((key, val))
        };

        self.inner.next().map(kv_decode)
    }
}

pub struct RocksIntoIterator<'a, S: StorageSchema, P: AsRef<[u8]>> {
    db:     Arc<DB>,
    column: &'a ColumnFamily,
    prefix: &'a P,
    pin_s:  PhantomData<S>,
}

impl<'a, 'b: 'a, S: StorageSchema, P: AsRef<[u8]>> IntoIterator
    for &'b RocksIntoIterator<'a, S, P>
{
    type IntoIter = StorageIterator<'a, S>;
    type Item = ProtocolResult<(<S as StorageSchema>::Key, <S as StorageSchema>::Value)>;

    fn into_iter(self) -> Self::IntoIter {
        let iter: DBIterator<'_> = self
            .db
            .prefix_iterator_cf(self.column, self.prefix.as_ref())
            .unwrap_or_else(|_| panic!("create db {:?} prefix iterator", hex_encode(self.prefix)));

        Box::new(RocksIterator {
            inner: iter,
            pin_s: PhantomData::<S>,
        })
    }
}

impl<'c, S: StorageSchema, P: AsRef<[u8]>> IntoIteratorByRef<S> for RocksIntoIterator<'c, S, P> {
    fn ref_to_iter<'a, 'b: 'a>(&'b self) -> StorageIterator<'a, S> {
        self.into_iter()
    }
}

#[async_trait]
impl StorageAdapter for RocksAdapter {
    async fn insert<S: StorageSchema>(&self, key: S::Key, val: S::Value) -> ProtocolResult<()> {
        let inst = Instant::now();

        let column = get_column::<S>(&self.db)?;
        let key = key.encode()?.to_vec();
        let val = val.encode()?.to_vec();
        let size = val.len() as i64;

        db!(self.db, put_cf, column, key, val)?;
        on_storage_put_cf(S::category(), inst.elapsed(), size as f64);

        Ok(())
    }

    async fn get<S: StorageSchema>(
        &self,
        key: <S as StorageSchema>::Key,
    ) -> ProtocolResult<Option<<S as StorageSchema>::Value>> {
        let column = get_column::<S>(&self.db)?;
        let key = key.encode()?;

        let opt_bytes =
            { db!(self.db, get_cf, column, key)?.map(|db_vec| Bytes::copy_from_slice(&db_vec)) };

        if let Some(bytes) = opt_bytes {
            let val = <_>::decode(bytes)?;

            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    async fn remove<S: StorageSchema>(&self, key: <S as StorageSchema>::Key) -> ProtocolResult<()> {
        let column = get_column::<S>(&self.db)?;
        let key = key.encode()?.to_vec();

        db!(self.db, delete_cf, column, key)?;

        Ok(())
    }

    async fn contains<S: StorageSchema>(
        &self,
        key: <S as StorageSchema>::Key,
    ) -> ProtocolResult<bool> {
        let column = get_column::<S>(&self.db)?;
        let key = key.encode()?.to_vec();
        let val = db!(self.db, get_cf, column, key)?;

        Ok(val.is_some())
    }

    async fn batch_modify<S: StorageSchema>(
        &self,
        keys: Vec<<S as StorageSchema>::Key>,
        vals: Vec<StorageBatchModify<S>>,
    ) -> ProtocolResult<()> {
        if keys.len() != vals.len() {
            return Err(RocksAdapterError::BatchLengthMismatch.into());
        }

        let column = get_column::<S>(&self.db)?;
        let mut pairs: Vec<(Bytes, Option<Bytes>)> = Vec::with_capacity(keys.len());

        for (key, value) in keys.into_iter().zip(vals.into_iter()) {
            let key = key.encode()?;

            let value = match value {
                StorageBatchModify::Insert(value) => Some(value.encode()?),
                StorageBatchModify::Remove => None,
            };

            pairs.push((key, value))
        }

        let mut batch = WriteBatch::default();
        let mut insert_size = 0usize;
        let inst = Instant::now();
        for (key, value) in pairs.into_iter() {
            match value {
                Some(value) => {
                    insert_size += value.len();
                    batch.put_cf(column, key, value)
                }
                None => batch.delete_cf(column, key),
            }
            .map_err(RocksAdapterError::from)?;
        }

        on_storage_put_cf(S::category(), inst.elapsed(), insert_size as f64);

        self.db.write(&batch).map_err(RocksAdapterError::from)?;
        Ok(())
    }

    fn prepare_iter<'a, 'b: 'a, S: StorageSchema + 'static, P: AsRef<[u8]> + 'a>(
        &'b self,
        prefix: &'a P,
    ) -> ProtocolResult<Box<dyn IntoIteratorByRef<S> + 'a>> {
        let column = get_column::<S>(&self.db)?;

        let rocks_iter = RocksIntoIterator {
            db: Arc::clone(&self.db),
            column,
            prefix,
            pin_s: PhantomData::<S>,
        };
        Ok(Box::new(rocks_iter))
    }
}

#[derive(Debug, Display, From)]
pub enum RocksAdapterError {
    #[display(fmt = "category {} not found", _0)]
    CategoryNotFound(&'static str),

    #[display(fmt = "rocksdb {}", _0)]
    RocksDB(rocksdb::Error),

    #[display(fmt = "parameters do not match")]
    InsertParameter,

    #[display(fmt = "batch length do not match")]
    BatchLengthMismatch,

    #[display(fmt = "Create DB path {}", _0)]
    CreateDB(io::Error),
}

impl Error for RocksAdapterError {}

impl From<RocksAdapterError> for ProtocolError {
    fn from(err: RocksAdapterError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Storage, Box::new(err))
    }
}

const C_BLOCKS: &str = "c1";
const C_BLOCK_HEADER: &str = "c2";
const C_SIGNED_TRANSACTIONS: &str = "c3";
const C_RECEIPTS: &str = "c4";
const C_WALS: &str = "c5";
const C_HASH_HEIGHT_MAP: &str = "c6";
const C_EVM_CODE_MAP: &str = "c7";

fn map_category(c: StorageCategory) -> &'static str {
    match c {
        StorageCategory::Block => C_BLOCKS,
        StorageCategory::BlockHeader => C_BLOCK_HEADER,
        StorageCategory::Receipt => C_RECEIPTS,
        StorageCategory::SignedTransaction => C_SIGNED_TRANSACTIONS,
        StorageCategory::Wal => C_WALS,
        StorageCategory::HashHeight => C_HASH_HEIGHT_MAP,
        StorageCategory::Code => C_EVM_CODE_MAP,
    }
}

fn get_column<S: StorageSchema>(db: &DB) -> Result<&ColumnFamily, RocksAdapterError> {
    let category = map_category(S::category());

    let column = db
        .cf_handle(category)
        .ok_or(RocksAdapterError::CategoryNotFound(category))?;

    Ok(column)
}
