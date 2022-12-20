use ckb_types::prelude::Entity;
use ckb_types::{bytes::Bytes, packed, prelude::*};
use rlp::{RlpDecodable, RlpEncodable};

use protocol::types::MerkleRoot;

use crate::system_contract::image_cell::error::ImageCellError;
use crate::system_contract::image_cell::trie_db::RocksTrieDB;
use crate::MPTTrie;

const BLOCK_NUMBER_KEY: &str = "BlockNumber";

#[derive(RlpEncodable, RlpDecodable)]
pub struct CellKey {
    pub tx_hash: Bytes,
    pub index:   u32,
}

#[derive(RlpEncodable, RlpDecodable)]
pub struct CellInfo {
    pub cell_output:     Bytes, // packed::CellOutput
    pub cell_data:       Bytes,
    pub created_number:  u64,
    pub consumed_number: Option<u64>,
}

#[derive(RlpEncodable, RlpDecodable)]
pub struct HeaderKey {
    pub block_number: u64,
    pub block_hash:   Bytes,
}

pub fn header_key(block_hash: &[u8; 32], block_number: u64) -> HeaderKey {
    HeaderKey {
        block_number,
        block_hash: block_hash.pack().as_bytes(),
    }
}

pub fn cell_key(tx_hash: &[u8; 32], index: u32) -> CellKey {
    CellKey {
        tx_hash: tx_hash.pack().as_bytes(),
        index,
    }
}

pub fn commit(mpt: &mut MPTTrie<RocksTrieDB>) -> Result<MerkleRoot, ImageCellError> {
    match mpt.commit() {
        Ok(root) => Ok(root),
        Err(e) => Err(ImageCellError::Protocol(e)),
    }
}

pub fn update_block_number(
    mpt: &mut MPTTrie<RocksTrieDB>,
    new_block_number: u64,
) -> Result<(), ImageCellError> {
    if let Err(e) = mpt.insert(BLOCK_NUMBER_KEY.as_bytes(), &rlp::encode(&new_block_number)) {
        return Err(ImageCellError::Protocol(e));
    }
    Ok(())
}

pub fn get_block_number(mpt: &MPTTrie<RocksTrieDB>) -> Result<Option<u64>, ImageCellError> {
    let block_number = match mpt.get(BLOCK_NUMBER_KEY.as_bytes()) {
        Ok(n) => match n {
            Some(n) => n,
            None => return Ok(None),
        },
        Err(e) => return Err(ImageCellError::Protocol(e)),
    };

    match rlp::decode(&block_number) {
        Ok(n) => Ok(Some(n)),
        Err(e) => Err(ImageCellError::RlpDecoder(e)),
    }
}

pub fn insert_header(
    mpt: &mut MPTTrie<RocksTrieDB>,
    key: &HeaderKey,
    header: &packed::Header,
) -> Result<(), ImageCellError> {
    if let Err(e) = mpt.insert(&rlp::encode(key), &header.as_bytes()) {
        return Err(ImageCellError::Protocol(e));
    }
    Ok(())
}

pub fn remove_header(
    mpt: &mut MPTTrie<RocksTrieDB>,
    key: &HeaderKey,
) -> Result<(), ImageCellError> {
    if let Err(e) = mpt.remove(&rlp::encode(key)) {
        return Err(ImageCellError::Protocol(e));
    }
    Ok(())
}

pub fn get_header(
    mpt: &MPTTrie<RocksTrieDB>,
    key: &HeaderKey,
) -> Result<Option<packed::Header>, ImageCellError> {
    let header = match mpt.get(&rlp::encode(key)) {
        Ok(n) => match n {
            Some(n) => n,
            None => return Ok(None),
        },
        Err(e) => return Err(ImageCellError::Protocol(e)),
    };

    match packed::Header::from_slice(&header) {
        Ok(h) => Ok(Some(h)),
        Err(e) => Err(ImageCellError::MoleculeVerification(e)),
    }
}

pub fn insert_cell(
    mpt: &mut MPTTrie<RocksTrieDB>,
    key: &CellKey,
    cell: &CellInfo,
) -> Result<(), ImageCellError> {
    if let Err(e) = mpt.insert(&rlp::encode(key), &rlp::encode(cell)) {
        return Err(ImageCellError::Protocol(e));
    }
    Ok(())
}

pub fn remove_cell(mpt: &mut MPTTrie<RocksTrieDB>, key: &CellKey) -> Result<(), ImageCellError> {
    if let Err(e) = mpt.remove(&rlp::encode(key)) {
        return Err(ImageCellError::Protocol(e));
    }
    Ok(())
}

pub fn get_cell(
    mpt: &MPTTrie<RocksTrieDB>,
    key: &CellKey,
) -> Result<Option<CellInfo>, ImageCellError> {
    let cell = match mpt.get(&rlp::encode(key)) {
        Ok(n) => match n {
            Some(n) => n,
            None => return Ok(None),
        },
        Err(e) => return Err(ImageCellError::Protocol(e)),
    };

    match rlp::decode(&cell) {
        Ok(h) => Ok(Some(h)),
        Err(e) => Err(ImageCellError::RlpDecoder(e)),
    }
}
