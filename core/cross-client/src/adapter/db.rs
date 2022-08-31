use std::{fs, path::Path, sync::Arc};

use sled::Db;

use common_config_parser::types::ConfigRocksDB;
use protocol::ProtocolResult;

use crate::{adapter::CrossChainDB, error::CrossChainError};

#[derive(Clone)]
pub struct CrossChainDBImpl {
    db: Arc<Db>,
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
        let mut ret: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();

        for item in self.db.iter() {
            let (key, val) = item.map_err(CrossChainError::from)?;
            ret.push(((*key).to_vec(), (*val).to_vec()))
        }

        Ok(ret)
    }

    fn insert(&self, key: &[u8], val: &[u8]) -> ProtocolResult<()> {
        self.db.insert(key, val).map_err(CrossChainError::from)?;
        Ok(())
    }

    fn remove(&self, key: &[u8]) -> ProtocolResult<()> {
        self.db.remove(key).map_err(CrossChainError::from)?;
        Ok(())
    }
}

impl CrossChainDBImpl {
    pub fn new<P: AsRef<Path>>(path: P, config: ConfigRocksDB) -> ProtocolResult<Self> {
        if !path.as_ref().is_dir() {
            fs::create_dir_all(&path).map_err(|_| CrossChainError::CreateDB)?;
        }

        Ok(CrossChainDBImpl {
            db: Arc::new(sled::open(path).map_err(CrossChainError::from)?),
        })
    }

    pub fn inner_db(&self) -> Arc<Db> {
        Arc::clone(&self.db)
    }
}
