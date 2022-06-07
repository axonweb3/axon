use std::fs;
use std::path::Path;
use std::sync::Arc;

use rocksdb::ops::{Delete, Get, Iterate, Open, Put};
use rocksdb::{FullOptions, IteratorMode, Options, WriteBatch, DB};

use common_config_parser::types::ConfigRocksDB;
use protocol::ProtocolResult;

use crate::adapter::CrossChainDB;
use crate::error::CrossChainError;

#[derive(Clone)]
pub struct CrossChainDBImpl {
    db: Arc<DB>,
}

impl CrossChainDB for CrossChainDBImpl {
    fn get(&self, key: &[u8]) -> ProtocolResult<Option<Vec<u8>>> {
        Ok(self
            .db
            .get(key)
            .map_err(CrossChainError::from)?
            .map(|r| r.to_vec()))
    }

    fn get_all(&self) -> ProtocolResult<Vec<(Vec<u8>, Vec<u8>)>> {
        Ok(self
            .db
            .iterator(IteratorMode::Start)
            .map(|(k, v)| (k.as_ref().to_vec(), v.as_ref().to_vec()))
            .collect())
    }

    fn insert(&self, key: &[u8], val: &[u8]) -> ProtocolResult<()> {
        self.db.put(key, val).map_err(CrossChainError::from)?;
        Ok(())
    }

    fn remove(&self, key: &[u8]) -> ProtocolResult<()> {
        self.db.delete(key).map_err(CrossChainError::from)?;
        Ok(())
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

    Ok(opts)
}
