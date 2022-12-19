use ckb_types::{bytes::Bytes, packed, prelude::*};
use rlp::{DecoderError, RlpDecodable, RlpEncodable};

use protocol::{Display, ProtocolError};

use crate::system_contract::image_cell::abi::image_cell_abi;
use crate::system_contract::image_cell::trie_db::RocksTrieDB;
use crate::system_contract::image_cell::BLOCK_NUMBER_KEY;
use crate::MPTTrie;

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

#[derive(Debug, Display)]
pub enum ImageCellError {
    #[display(fmt = "Invalid block number: {:?}", _0)]
    InvalidBlockNumber(u64),

    #[display(fmt = "RLP decoding failed: {:?}", _0)]
    RlpDecoder(DecoderError),

    #[display(fmt = "Protocol error: {:?}", _0)]
    Protocol(ProtocolError),
}

// todo: only contract cells will be saved in the cache
pub fn update(
    mpt: &mut MPTTrie<RocksTrieDB>,
    data: image_cell_abi::UpdateCall,
) -> Result<(), ImageCellError> {
    let new_block_number = data.header.number;

    check_block_number_updated(mpt, new_block_number)?;

    save_cells(mpt, data.outputs, new_block_number)?;

    mark_cells_consumed(mpt, data.inputs, new_block_number)?;

    save_header(mpt, &data.header, new_block_number)?;

    update_block_number(mpt, new_block_number)?;

    Ok(())
}

pub fn rollback(
    mpt: &mut MPTTrie<RocksTrieDB>,
    data: image_cell_abi::RollbackCall,
) -> Result<(), ImageCellError> {
    let cur_block_number = data.block_number;
    let new_block_number = cur_block_number - 1;

    check_block_number_rolled(mpt, cur_block_number)?;

    remove_cells(mpt, data.outputs)?;

    mark_cells_not_consumed(mpt, data.inputs)?;

    remove_header(mpt, cur_block_number, &data.block_hash)?;

    update_block_number(mpt, new_block_number)?;

    Ok(())
}

fn check_block_number_updated(
    mpt: &MPTTrie<RocksTrieDB>,
    new_block_number: u64,
) -> Result<(), ImageCellError> {
    if let Some(cur_block_number) = get_block_number(mpt)? {
        if new_block_number != cur_block_number + 1 {
            return Err(ImageCellError::InvalidBlockNumber(new_block_number));
        }
    }
    Ok(())
}

pub fn get_block_number(mpt: &MPTTrie<RocksTrieDB>) -> Result<Option<u64>, ImageCellError> {
    match mpt.get(BLOCK_NUMBER_KEY.as_bytes()) {
        Ok(n) => match n {
            Some(n) => match rlp::decode(&n) {
                Ok(n) => Ok(Some(n)),
                Err(e) => Err(ImageCellError::RlpDecoder(e)),
            },
            None => Ok(None),
        },
        Err(e) => Err(ImageCellError::Protocol(e)),
    }
}

fn save_cells(
    mpt: &mut MPTTrie<RocksTrieDB>,
    outputs: Vec<image_cell_abi::CellInfo>,
    new_block_number: u64,
) -> Result<(), ImageCellError> {
    for cell in outputs {
        let key = CellKey {
            tx_hash: cell.out_point.tx_hash.pack().as_bytes(),
            index:   cell.out_point.index,
        };

        let lock = cell.output.lock;
        let lock = packed::Script::new_builder()
            .args(lock.args.0.pack())
            .code_hash(lock.code_hash.pack())
            .hash_type(lock.hash_type.into())
            .build();

        let type_builder = packed::ScriptOpt::new_builder();
        let type_ = if !cell.output.type_.is_empty() {
            let type_ = &cell.output.type_[0];
            let type_ = packed::Script::new_builder()
                .args(type_.args.0.pack())
                .code_hash(type_.code_hash.pack())
                .hash_type(type_.hash_type.into())
                .build();
            type_builder.set(Some(type_))
        } else {
            type_builder
        }
        .build();

        let cell_output = packed::CellOutput::new_builder()
            .lock(lock)
            .type_(type_)
            .capacity(cell.output.capacity.pack())
            .build();

        let cell = CellInfo {
            cell_output:     cell_output.as_bytes(),
            cell_data:       cell.data.0,
            created_number:  new_block_number,
            consumed_number: None,
        };

        if let Err(e) = mpt.insert(&rlp::encode(&key), &rlp::encode(&cell)) {
            return Err(ImageCellError::Protocol(e));
        }
    }
    Ok(())
}

fn mark_cells_consumed(
    mpt: &mut MPTTrie<RocksTrieDB>,
    inputs: Vec<image_cell_abi::OutPoint>,
    new_block_number: u64,
) -> Result<(), ImageCellError> {
    for input in inputs {
        let key = CellKey {
            tx_hash: input.tx_hash.pack().as_bytes(),
            index:   input.index,
        };

        if let Some(ref mut cell) = get_cell(mpt, &key)? {
            cell.consumed_number = Some(new_block_number);
            insert_cell(mpt, &key, cell)?;
        }
    }
    Ok(())
}

fn get_cell(mpt: &MPTTrie<RocksTrieDB>, key: &CellKey) -> Result<Option<CellInfo>, ImageCellError> {
    let cell = match mpt.get(&rlp::encode(key)) {
        Ok(c) => match c {
            Some(c) => c,
            None => return Ok(None),
        },
        Err(e) => return Err(ImageCellError::Protocol(e)),
    };

    let cell: CellInfo = match rlp::decode(&cell) {
        Ok(c) => c,
        Err(e) => return Err(ImageCellError::RlpDecoder(e)),
    };

    Ok(Some(cell))
}

fn insert_cell(
    mpt: &mut MPTTrie<RocksTrieDB>,
    key: &CellKey,
    cell: &CellInfo,
) -> Result<(), ImageCellError> {
    if let Err(e) = mpt.insert(&rlp::encode(key), &rlp::encode(cell)) {
        return Err(ImageCellError::Protocol(e));
    }
    Ok(())
}

fn save_header(
    mpt: &mut MPTTrie<RocksTrieDB>,
    header: &image_cell_abi::Header,
    new_block_number: u64,
) -> Result<(), ImageCellError> {
    let header_key = HeaderKey {
        block_number: new_block_number,
        block_hash:   header.block_hash.pack().as_bytes(),
    };

    let raw = packed::RawHeader::new_builder()
        .compact_target(header.compact_target.pack())
        .dao(header.dao.pack())
        .epoch(header.epoch.pack())
        .extra_hash(header.block_hash.pack())
        .number(header.number.pack())
        .parent_hash(header.parent_hash.pack())
        .proposals_hash(header.proposals_hash.pack())
        .timestamp(header.timestamp.pack())
        .transactions_root(header.transactions_root.pack())
        .version(header.version.pack())
        .build();

    let packed_header = packed::Header::new_builder()
        .raw(raw)
        .nonce(header.nonce.pack())
        .build();

    if let Err(e) = mpt.insert(&rlp::encode(&header_key), &packed_header.as_bytes()) {
        return Err(ImageCellError::Protocol(e));
    }

    Ok(())
}

fn update_block_number(
    mpt: &mut MPTTrie<RocksTrieDB>,
    new_block_number: u64,
) -> Result<(), ImageCellError> {
    if let Err(e) = mpt.insert(BLOCK_NUMBER_KEY.as_bytes(), &rlp::encode(&new_block_number)) {
        return Err(ImageCellError::Protocol(e));
    }
    Ok(())
}

fn check_block_number_rolled(
    mpt: &MPTTrie<RocksTrieDB>,
    cur_block_number: u64,
) -> Result<(), ImageCellError> {
    if let Some(block_number) = get_block_number(mpt)? {
        if block_number != cur_block_number {
            return Err(ImageCellError::InvalidBlockNumber(cur_block_number));
        }
    }
    Ok(())
}

fn remove_cells(
    mpt: &mut MPTTrie<RocksTrieDB>,
    outputs: Vec<image_cell_abi::OutPoint>,
) -> Result<(), ImageCellError> {
    for output in outputs {
        let key = CellKey {
            tx_hash: output.tx_hash.pack().as_bytes(),
            index:   output.index,
        };

        if let Err(e) = mpt.remove(&rlp::encode(&key)) {
            return Err(ImageCellError::Protocol(e));
        }
    }
    Ok(())
}

fn mark_cells_not_consumed(
    mpt: &mut MPTTrie<RocksTrieDB>,
    inputs: Vec<image_cell_abi::OutPoint>,
) -> Result<(), ImageCellError> {
    for input in inputs {
        let key = CellKey {
            tx_hash: input.tx_hash.pack().as_bytes(),
            index:   input.index,
        };

        if let Some(ref mut cell) = get_cell(mpt, &key)? {
            cell.consumed_number = None;
            insert_cell(mpt, &key, cell)?;
        }
    }
    Ok(())
}

fn remove_header(
    mpt: &mut MPTTrie<RocksTrieDB>,
    block_number: u64,
    block_hash: &[u8; 32],
) -> Result<(), ImageCellError> {
    let header_key = HeaderKey {
        block_number,
        block_hash: block_hash.pack().as_bytes(),
    };

    if let Err(e) = mpt.remove(&rlp::encode(&header_key)) {
        return Err(ImageCellError::Protocol(e));
    }

    Ok(())
}
