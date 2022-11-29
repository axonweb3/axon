use std::error::Error;
use std::iter::FromIterator;
use std::sync::Arc;

use hasher::HasherKeccak;
use static_merkle_tree::Tree;

use protocol::codec::ProtocolCodec;
use protocol::trie::{MemoryDB, PatriciaTrie, Trie};
use protocol::types::{Bytes, Hash, Hasher};

#[derive(Debug, Clone)]
pub struct ProofNode {
    pub is_right: bool,
    pub hash:     Hash,
}

pub struct Merkle {
    tree: Tree<Hash>,
}

impl Merkle {
    pub fn from_hashes(hashes: Vec<Hash>) -> Self {
        let tree = Tree::from_hashes(hashes, merge);
        Merkle { tree }
    }

    pub fn get_root_hash(&self) -> Option<Hash> {
        self.tree.get_root_hash().copied()
    }

    pub fn get_proof_by_input_index(&self, input_index: usize) -> Option<Vec<ProofNode>> {
        self.tree
            .get_proof_by_input_index(input_index)
            .map(|proof| {
                proof
                    .0
                    .into_iter()
                    .map(|node| ProofNode {
                        is_right: node.is_right,
                        hash:     node.hash,
                    })
                    .collect()
            })
    }
}

fn merge(left: &Hash, right: &Hash) -> Hash {
    let left = left.as_bytes();
    let right = right.as_bytes();

    let mut root = Vec::with_capacity(left.len() + right.len());
    root.extend_from_slice(left);
    root.extend_from_slice(right);
    Hasher::digest(Bytes::from(root))
}

pub struct TrieMerkle(PatriciaTrie<MemoryDB, HasherKeccak>);

impl Default for TrieMerkle {
    fn default() -> Self {
        TrieMerkle(PatriciaTrie::new(
            Arc::new(MemoryDB::new(false)),
            Arc::new(HasherKeccak::new()),
        ))
    }
}

impl<'a, C: ProtocolCodec + 'a> FromIterator<(usize, &'a C)> for TrieMerkle {
    fn from_iter<T: IntoIterator<Item = (usize, &'a C)>>(iter: T) -> Self {
        let mut trie = Self::default();

        iter.into_iter().for_each(|(i, val)| {
            trie.0
                .insert(rlp::encode(&i).to_vec(), val.encode().unwrap().to_vec())
                .unwrap()
        });

        trie
    }
}

impl TrieMerkle {
    pub fn root_hash(&mut self) -> Result<Hash, Box<dyn Error + 'static>> {
        Ok(Hash::from_slice(&self.0.root()?))
    }

    pub fn get_proof_by_index(
        &self,
        index: usize,
    ) -> Result<Vec<Vec<u8>>, Box<dyn Error + 'static>> {
        let key = rlp::encode(&index).to_vec();
        let ret = self.0.get_proof(&key)?;
        Ok(ret)
    }
}
