use ckb_types::prelude::Entity;
use ckb_types::{bytes::Bytes, packed};
use rlp::{RlpDecodable, RlpEncodable};

use protocol::types::{MerkleRoot, H256};

use crate::system_contract::error::{SystemScriptError, SystemScriptResult};
use crate::system_contract::image_cell::trie_db::RocksTrieDB;
use crate::system_contract::image_cell::MPTTrie;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellKey {
    pub tx_hash: H256,
    pub index:   u32,
}

impl CellKey {
    const ENCODED_LEN: usize = 32 + 4;

    pub fn new(tx_hash: [u8; 32], index: u32) -> Self {
        let tx_hash = H256(tx_hash);
        CellKey { tx_hash, index }
    }

    pub fn decode(input: &[u8]) -> SystemScriptResult<Self> {
        if input.len() != Self::ENCODED_LEN {
            return Err(SystemScriptError::DataLengthMismatch {
                expect: Self::ENCODED_LEN,
                actual: input.len(),
            });
        }

        let tx_hash = H256::from_slice(&input[0..32]);
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&input[32..36]);
        let index = u32::from_le_bytes(buf);

        Ok(CellKey { tx_hash, index })
    }

    pub fn encode(&self) -> Bytes {
        let mut ret = Vec::with_capacity(Self::ENCODED_LEN);
        ret.extend_from_slice(&self.tx_hash.0);
        ret.extend_from_slice(&self.index.to_le_bytes());
        ret.into()
    }
}

#[derive(RlpEncodable, RlpDecodable)]
pub struct CellInfo {
    pub cell_output:     Bytes, // packed::CellOutput
    pub cell_data:       Bytes,
    pub created_number:  u64,
    pub consumed_number: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderKey {
    pub block_number: u64,
    pub block_hash:   H256,
}

impl HeaderKey {
    const ENCODED_LEN: usize = 8 + 32;

    pub fn new(block_hash: [u8; 32], block_number: u64) -> Self {
        let block_hash = H256(block_hash);
        HeaderKey {
            block_number,
            block_hash,
        }
    }

    pub fn decode(input: &[u8]) -> SystemScriptResult<Self> {
        if input.len() != Self::ENCODED_LEN {
            return Err(SystemScriptError::DataLengthMismatch {
                expect: Self::ENCODED_LEN,
                actual: input.len(),
            });
        }

        let block_hash = H256::from_slice(&input[8..40]);
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&input[0..8]);
        let block_number = u64::from_le_bytes(buf);

        Ok(HeaderKey {
            block_number,
            block_hash,
        })
    }

    pub fn encode(&self) -> Bytes {
        let mut ret = Vec::with_capacity(Self::ENCODED_LEN);
        ret.extend_from_slice(&self.block_number.to_le_bytes());
        ret.extend_from_slice(&self.block_hash.0);
        ret.into()
    }
}

pub fn commit(mpt: &mut MPTTrie<RocksTrieDB>) -> SystemScriptResult<MerkleRoot> {
    mpt.commit()
        .map_err(|e| SystemScriptError::CommitError(e.to_string()))
}

pub fn insert_header(
    mpt: &mut MPTTrie<RocksTrieDB>,
    key: &HeaderKey,
    header: &packed::Header,
) -> SystemScriptResult<()> {
    mpt.insert(&key.encode(), &header.as_bytes())
        .map_err(|e| SystemScriptError::InsertHeader(e.to_string()))
}

pub fn remove_header(mpt: &mut MPTTrie<RocksTrieDB>, key: &HeaderKey) -> SystemScriptResult<()> {
    mpt.remove(&key.encode())
        .map_err(|e| SystemScriptError::RemoveHeader(e.to_string()))
}

pub fn get_header(
    mpt: &MPTTrie<RocksTrieDB>,
    key: &HeaderKey,
) -> SystemScriptResult<Option<packed::Header>> {
    let header = match mpt.get(&key.encode()) {
        Ok(n) => match n {
            Some(n) => n,
            None => return Ok(None),
        },
        Err(e) => return Err(SystemScriptError::GetHeader(e.to_string())),
    };

    Ok(Some(
        packed::Header::from_slice(&header).map_err(SystemScriptError::MoleculeVerification)?,
    ))
}

pub fn insert_cell(
    mpt: &mut MPTTrie<RocksTrieDB>,
    key: &CellKey,
    cell: &CellInfo,
) -> SystemScriptResult<()> {
    mpt.insert(&key.encode(), &rlp::encode(cell))
        .map_err(|e| SystemScriptError::InsertCell(e.to_string()))
}

pub fn remove_cell(mpt: &mut MPTTrie<RocksTrieDB>, key: &CellKey) -> SystemScriptResult<()> {
    mpt.remove(&key.encode())
        .map_err(|e| SystemScriptError::RemoveCell(e.to_string()))
}

pub fn get_cell(mpt: &MPTTrie<RocksTrieDB>, key: &CellKey) -> SystemScriptResult<Option<CellInfo>> {
    let cell = match mpt.get(&key.encode()) {
        Ok(n) => match n {
            Some(n) => n,
            None => return Ok(None),
        },
        Err(e) => return Err(SystemScriptError::GetCell(e.to_string())),
    };

    Ok(Some(
        rlp::decode(&cell).map_err(SystemScriptError::DecodeCell)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random;

    #[test]
    fn test_key_codec() {
        for _ in 0..10 {
            let cell_key = CellKey {
                tx_hash: H256::random(),
                index:   random(),
            };
            let header_key = HeaderKey {
                block_number: random(),
                block_hash:   H256::random(),
            };

            assert_eq!(CellKey::decode(&cell_key.encode()).unwrap(), cell_key);
            assert_eq!(HeaderKey::decode(&header_key.encode()).unwrap(), header_key);
        }
    }
}
