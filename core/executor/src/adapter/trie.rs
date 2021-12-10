use std::sync::Arc;

use cita_trie::{PatriciaTrie, Trie, TrieError, DB as TrieDB};
use hasher::HasherKeccak;

use protocol::types::{Bytes, Hash, MerkleRoot};
use protocol::{Display, From, ProtocolError, ProtocolErrorKind, ProtocolResult};

lazy_static::lazy_static! {
    static ref HASHER_INST: Arc<HasherKeccak> = Arc::new(HasherKeccak::new());
}

pub struct MPTTrie<DB: TrieDB> {
    root: MerkleRoot,
    trie: PatriciaTrie<DB, HasherKeccak>,
}

impl<DB: TrieDB> MPTTrie<DB> {
    pub fn _new(db: Arc<DB>) -> Self {
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

#[derive(Debug, Display, From)]
pub enum MPTTrieError {
    #[display(fmt = "{:?}", _0)]
    Trie(TrieError),

    #[display(fmt = "Remove failed")]
    RemoveFailed,
}

impl std::error::Error for MPTTrieError {}

impl From<MPTTrieError> for ProtocolError {
    fn from(err: MPTTrieError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Executor, Box::new(err))
    }
}
