use std::sync::Arc;

use hasher::HasherKeccak;

use protocol::trie::{PatriciaTrie, Trie, TrieError, DB as TrieDB};
use protocol::types::{Hasher, MerkleRoot};
use protocol::ProtocolResult;

pub struct MPTTrie<DB: TrieDB>(PatriciaTrie<DB, HasherKeccak>);

impl<DB: TrieDB> Trie<DB, HasherKeccak> for MPTTrie<DB> {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, TrieError> {
        self.0.get(&Hasher::digest(key).0)
    }

    fn contains(&self, key: &[u8]) -> Result<bool, TrieError> {
        self.0.contains(&Hasher::digest(key).0)
    }

    fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), TrieError> {
        self.0.insert(Hasher::digest(key).0.to_vec(), value)
    }

    fn remove(&mut self, key: &[u8]) -> Result<bool, TrieError> {
        self.0.remove(&Hasher::digest(key).0)
    }

    fn root(&mut self) -> Result<Vec<u8>, TrieError> {
        self.0.root()
    }

    fn get_proof(&self, key: &[u8]) -> Result<Vec<Vec<u8>>, TrieError> {
        self.0.get_proof(&Hasher::digest(key).0)
    }

    fn verify_proof(
        &self,
        root_hash: &[u8],
        key: &[u8],
        proof: Vec<Vec<u8>>,
    ) -> Result<Option<Vec<u8>>, TrieError> {
        self.0
            .verify_proof(root_hash, &Hasher::digest(key).0, proof)
    }
}

impl<DB: TrieDB> MPTTrie<DB> {
    pub fn new(db: Arc<DB>) -> Self {
        MPTTrie(PatriciaTrie::new(db, Arc::new(HasherKeccak::new())))
    }

    pub fn from_root(root: MerkleRoot, db: Arc<DB>) -> ProtocolResult<Self> {
        Ok(MPTTrie(PatriciaTrie::from(
            db,
            Arc::new(HasherKeccak::new()),
            root.as_bytes(),
        )?))
    }

    pub fn commit(&mut self) -> ProtocolResult<MerkleRoot> {
        self.0
            .root()
            .map(|r| MerkleRoot::from_slice(&r))
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core_db::RocksAdapter;
    use protocol::rand::random;

    use crate::adapter::RocksTrieDB;

    fn rand_bytes(len: usize) -> Vec<u8> {
        (0..len).map(|_| random()).collect()
    }

    #[test]
    fn test_mpt_cache() {
        let dir = tempfile::tempdir().unwrap();
        let inner_db =
            Arc::new(RocksAdapter::new(dir.path(), Default::default()).unwrap()).inner_db();
        let db = RocksTrieDB::new_evm(inner_db, 100);
        let mut mpt = MPTTrie::new(Arc::new(db));

        let key_1 = rand_bytes(5);
        let val_1 = rand_bytes(10);
        let key_2 = rand_bytes(10);
        let val_2 = rand_bytes(20);

        mpt.insert(key_1.clone(), val_1.clone()).unwrap();
        mpt.insert(key_2.clone(), val_2.clone()).unwrap();
        mpt.commit().unwrap();

        assert_eq!(mpt.get(&key_1).unwrap(), Some(val_1));
        assert_eq!(mpt.get(&key_2).unwrap(), Some(val_2));
        assert!(mpt.remove(&key_1).is_ok());
        assert!(mpt.get(&key_1).unwrap().is_none());

        mpt.commit().unwrap();

        assert!(mpt.get(&key_1).unwrap().is_none());

        dir.close().unwrap();
    }
}
