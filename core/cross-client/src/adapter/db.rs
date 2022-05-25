use rocksdb::ops::{Get, Open, Put, WriteOps};
use rocksdb::{FullOptions, Options, WriteBatch, DB};

use common_config_parser::types::ConfigRocksDB;
use protocol::types::Hash;
use protocol::ProtocolResult;

use crate::adapter::CrossChainDB;
use crate::error::CrossChainError;

#[derive(Clone)]
pub struct CrossChainDBImpl {
    db: Arc<DB>,
}

impl CrossChainDB for CrossChainDBImpl {
    type Error = CrossChainError;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.db.get(key)?.map(|r| r.to_vec()))
    }

    fn insert_determine_record(
        &self,
        direct: u8,
        origin_tx_hash: Hash,
        relay_tx_hash: Hash,
    ) -> Result<(), Self::Error> {
        let mut key = vec![direct];
        key.extend_from_slice(origin_tx_hash.as_bytes());
        self.db.put(&key, relay_tx_hash.as_bytes())?;
        Ok(())
    }

    fn insert_batch_record(
        &self,
        direct: u8,
        origin_tx_hashes: Vec<Hash>,
        relay_tx_hashes: Vec<Hash>,
    ) -> Result<(), Self::Error> {
        let base = vec![direct];
        if origin_tx_hashes.len() != relay_tx_hashes.len() {
            return Err(CrossChainError::BatchLengthMismatch);
        }

        let mut batch = WriteBatch::default();
        for (origin_hash, relay_hash) in origin_tx_hashes.iter().zip(relay_tx_hashes.iter()) {
            let mut key = base.clone();
            key.extend_from_slice(origin_hash.as_bytes());
            batch.put(&key, relay_hash.as_bytes())?;
        }

        self.db.write(&batch).map_err(Into::into)
    }

    fn insert_pending_request(&self, key: &[u8], val: &[u8]) -> Result<(), Self::Error> {
        self.db.put(key, val).map_err(Into::into)
    }

    fn remove_pending_request(&self, key: &[u8]) -> Result<(), Self::Error> {
        self.db.delete(key).map_err(Into::into)
    }
}

impl CrossChainDBImpl {
    pub fn new<P: AsRef<Path>>(
        path: P,
        config: ConfigRocksDB,
        cache_size: usize,
    ) -> ProtocolResult<Self> {
        if !path.as_ref().is_dir() {
            fs::create_dir_all(&path).map_err(|_| CrossChainError::CreateDB)?;
        }

        let opts = rocksdb_opts(config)?;

        Ok(CrossChainDBImpl {
            db: Arc::new(DB::open(&opts, path).map_err(CrossChainError::from)?),
        })
    }

    pub fn inner_db(&self) -> Arc<DB> {
        Arc::clone(&self.db)
    }
}

fn rocksdb_opts(config: ConfigRocksDB) -> ProtocolResult<Options> {
    let mut opts = if let Some(ref file) = config.options_file {
        let cache_size = match config.cache_size {
            0 => None,
            size => Some(size),
        };
        let full_opts =
            FullOptions::load_from_file(file, cache_size, false).map_err(CrossChainError::from)?;
        let FullOptions { db_opts, .. } = full_opts;
        db_opts
    } else {
        Options::default()
    };

    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_max_open_files(config.max_open_files);

