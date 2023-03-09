use ckb_types::{packed, prelude::*};

use protocol::types::MerkleRoot;
use protocol::ProtocolResult;

use crate::system_contract::image_cell::store::{
    commit, get_cell, insert_cell, remove_cell, CellInfo,
};
use crate::system_contract::image_cell::{abi::image_cell_abi, trie_db::RocksTrieDB};
use crate::system_contract::image_cell::{CellKey, MPTTrie};

pub fn update(
    mpt: &mut MPTTrie<RocksTrieDB>,
    data: image_cell_abi::UpdateCall,
) -> ProtocolResult<MerkleRoot> {
    save_cells(mpt, data.outputs, data.block_number)?;

    mark_cells_consumed(mpt, data.inputs, data.block_number)?;

    commit(mpt)
}

pub fn rollback(
    mpt: &mut MPTTrie<RocksTrieDB>,
    data: image_cell_abi::RollbackCall,
) -> ProtocolResult<MerkleRoot> {
    remove_cells(mpt, data.outputs)?;

    mark_cells_not_consumed(mpt, data.inputs)?;

    commit(mpt)
}

pub(crate) fn save_cells(
    mpt: &mut MPTTrie<RocksTrieDB>,
    outputs: Vec<image_cell_abi::CellInfo>,
    created_number: u64,
) -> ProtocolResult<()> {
    for cell in outputs {
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

        let cell_info = CellInfo {
            cell_output: cell_output.as_bytes(),
            cell_data: cell.data.0,
            created_number,
            consumed_number: None,
        };

        let key = CellKey::new(cell.out_point.tx_hash, cell.out_point.index);

        insert_cell(mpt, &key, &cell_info)?;
    }
    Ok(())
}

fn mark_cells_consumed(
    mpt: &mut MPTTrie<RocksTrieDB>,
    inputs: Vec<image_cell_abi::OutPoint>,
    new_block_number: u64,
) -> ProtocolResult<()> {
    for input in inputs {
        let key = CellKey::new(input.tx_hash, input.index);

        if let Some(ref mut cell) = get_cell(mpt, &key)? {
            cell.consumed_number = Some(new_block_number);
            insert_cell(mpt, &key, cell)?;
        }
    }
    Ok(())
}

fn remove_cells(
    mpt: &mut MPTTrie<RocksTrieDB>,
    outputs: Vec<image_cell_abi::OutPoint>,
) -> ProtocolResult<()> {
    for output in outputs {
        let key = CellKey::new(output.tx_hash, output.index);
        remove_cell(mpt, &key)?;
    }
    Ok(())
}

fn mark_cells_not_consumed(
    mpt: &mut MPTTrie<RocksTrieDB>,
    inputs: Vec<image_cell_abi::OutPoint>,
) -> ProtocolResult<()> {
    for input in inputs {
        let key = CellKey::new(input.tx_hash, input.index);
        if let Some(ref mut cell) = get_cell(mpt, &key)? {
            cell.consumed_number = None;
            insert_cell(mpt, &key, cell)?;
        }
    }
    Ok(())
}
