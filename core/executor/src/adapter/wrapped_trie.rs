use std::sync::Arc;

use hasher::HasherKeccak;

use protocol::trie::{PatriciaTrie, Trie, TrieError, DB as TrieDB};
use protocol::types::{Bytes, MerkleRoot};
use protocol::{
    codec::hex_encode, Display, From, ProtocolError, ProtocolErrorKind, ProtocolResult,
};

pub struct MPTTrie<DB: TrieDB>(PatriciaTrie<DB, HasherKeccak>);

impl<DB: TrieDB> MPTTrie<DB> {
    pub fn new(db: Arc<DB>) -> Self {
        MPTTrie(PatriciaTrie::new(db, Arc::new(HasherKeccak::new())))
    }

    pub fn from_root(root: MerkleRoot, db: Arc<DB>) -> ProtocolResult<Self> {
        Ok(MPTTrie(
            PatriciaTrie::from(db, Arc::new(HasherKeccak::new()), root.as_bytes())
                .map_err(MPTTrieError::from)?,
        ))
    }

    pub fn get(&self, key: &[u8]) -> ProtocolResult<Option<Bytes>> {
        Ok(self
            .0
            .get(key)
            .map_err(MPTTrieError::from)?
            .map(Bytes::from))
    }

    pub fn contains(&self, key: &[u8]) -> ProtocolResult<bool> {
        Ok(self.0.contains(key).map_err(MPTTrieError::from)?)
    }

    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> ProtocolResult<()> {
        self.0
            .insert(key.to_vec(), value.to_vec())
            .map_err(MPTTrieError::from)?;
        Ok(())
    }

    pub fn remove(&mut self, key: &[u8]) -> ProtocolResult<()> {
        if self.0.remove(key).map_err(MPTTrieError::from)? {
            Ok(())
        } else {
            Err(MPTTrieError::RemoveFailed(hex_encode(key)).into())
        }
    }

    pub fn commit(&mut self) -> ProtocolResult<MerkleRoot> {
        Ok(MerkleRoot::from_slice(
            &self.0.root().map_err(MPTTrieError::from)?,
        ))
    }
}

#[derive(Debug, Display, From)]
pub enum MPTTrieError {
    #[display(fmt = "Trie {:?}", _0)]
    Trie(TrieError),

    #[display(fmt = "Remove {:?} failed", _0)]
    RemoveFailed(String),
}

impl std::error::Error for MPTTrieError {}

impl From<MPTTrieError> for ProtocolError {
    fn from(err: MPTTrieError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Executor, Box::new(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::RocksTrieDB;
    use getrandom::getrandom;

    fn rand_bytes(len: usize) -> Vec<u8> {
        let mut ret = (0..len).map(|_| 0u8).collect::<Vec<_>>();
        getrandom(&mut ret).unwrap();
        ret
    }

    #[test]
    fn test_mpt_cache() {
        let dir = tempfile::tempdir().unwrap();
        let db = RocksTrieDB::new(dir.path(), Default::default(), 100).unwrap();
        let mut mpt = MPTTrie::new(Arc::new(db));

        let key_1 = rand_bytes(5);
        let val_1 = rand_bytes(10);
        let key_2 = rand_bytes(10);
        let val_2 = rand_bytes(20);

        mpt.insert(&key_1, &val_1).unwrap();
        mpt.insert(&key_2, &val_2).unwrap();
        mpt.commit().unwrap();

        assert_eq!(mpt.get(&key_1).unwrap(), Some(Bytes::from(val_1)));
        assert_eq!(mpt.get(&key_2).unwrap(), Some(Bytes::from(val_2)));
        assert!(mpt.remove(&key_1).is_ok());
        assert!(mpt.get(&key_1).unwrap().is_none());

        mpt.commit().unwrap();

        assert!(mpt.get(&key_1).unwrap().is_none());

        dir.close().unwrap();
    }
}
