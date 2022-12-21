use ckb_types::prelude::Entity;
use ckb_types::{bytes::Bytes, packed, prelude::*};
use rlp::{RlpDecodable, RlpEncodable};

use protocol::types::MerkleRoot;

use crate::system_contract::image_cell::error::{ImageCellError, ImageCellResult};
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

pub fn commit(mpt: &mut MPTTrie<RocksTrieDB>) -> ImageCellResult<MerkleRoot> {
    mpt.commit().map_err(ImageCellError::CommitError)
}

pub fn update_block_number(
    mpt: &mut MPTTrie<RocksTrieDB>,
    new_block_number: u64,
) -> ImageCellResult<()> {
    mpt.insert(BLOCK_NUMBER_KEY.as_bytes(), &rlp::encode(&new_block_number))
        .map_err(|e| ImageCellError::UpdateBlockNumber {
            e,
            number: new_block_number,
        })
}

pub fn get_block_number(mpt: &MPTTrie<RocksTrieDB>) -> ImageCellResult<Option<u64>> {
    let block_number = match mpt.get(BLOCK_NUMBER_KEY.as_bytes()) {
        Ok(n) => match n {
            Some(n) => n,
            None => return Ok(None),
        },
        Err(e) => return Err(ImageCellError::GetBlockNumber(e)),
    };

    Ok(Some(
        rlp::decode(&block_number).map_err(ImageCellError::RlpDecodeBlockNumber)?,
    ))
}

pub fn insert_header(
    mpt: &mut MPTTrie<RocksTrieDB>,
    key: &HeaderKey,
    header: &packed::Header,
) -> ImageCellResult<()> {
    mpt.insert(&rlp::encode(key), &header.as_bytes())
        .map_err(ImageCellError::InsertHeader)
}

pub fn remove_header(mpt: &mut MPTTrie<RocksTrieDB>, key: &HeaderKey) -> ImageCellResult<()> {
    mpt.remove(&rlp::encode(key))
        .map_err(ImageCellError::RemoveHeader)
}

pub fn get_header(
    mpt: &MPTTrie<RocksTrieDB>,
    key: &HeaderKey,
) -> ImageCellResult<Option<packed::Header>> {
    let header = match mpt.get(&rlp::encode(key)) {
        Ok(n) => match n {
            Some(n) => n,
            None => return Ok(None),
        },
        Err(e) => return Err(ImageCellError::GetHeader(e)),
    };

    Ok(Some(
        packed::Header::from_slice(&header).map_err(ImageCellError::MoleculeVerification)?,
    ))
}

pub fn insert_cell(
    mpt: &mut MPTTrie<RocksTrieDB>,
    key: &CellKey,
    cell: &CellInfo,
) -> ImageCellResult<()> {
    mpt.insert(&rlp::encode(key), &rlp::encode(cell))
        .map_err(ImageCellError::InsertCell)
}

pub fn remove_cell(mpt: &mut MPTTrie<RocksTrieDB>, key: &CellKey) -> ImageCellResult<()> {
    mpt.remove(&rlp::encode(key))
        .map_err(ImageCellError::RemoveCell)
}

pub fn get_cell(mpt: &MPTTrie<RocksTrieDB>, key: &CellKey) -> ImageCellResult<Option<CellInfo>> {
    let cell = match mpt.get(&rlp::encode(key)) {
        Ok(n) => match n {
            Some(n) => n,
            None => return Ok(None),
        },
        Err(e) => return Err(ImageCellError::GetCell(e)),
    };

    Ok(Some(
        rlp::decode(&cell).map_err(ImageCellError::RlpDecodeCell)?,
    ))
}
