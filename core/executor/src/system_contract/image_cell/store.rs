use std::sync::Arc;

use ckb_types::{bytes::Bytes, core::cell::CellMeta, packed, prelude::*};
use rlp::{RlpDecodable, RlpEncodable};

use protocol::ckb_blake2b_256;
use protocol::types::H256;
use protocol::ProtocolResult;

use crate::system_contract::image_cell::{image_cell_abi, MPTTrie};
use crate::system_contract::HEADER_CELL_DB;
use crate::{
    adapter::RocksTrieDB, system_contract::error::SystemScriptError, CURRENT_HEADER_CELL_ROOT,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellKey {
    pub tx_hash: H256,
    pub index:   u32,
}

pub struct ImageCellStore {
    pub trie: MPTTrie<RocksTrieDB>,
}

impl ImageCellStore {
    pub fn new() -> ProtocolResult<Self> {
        let trie_db = match HEADER_CELL_DB.get() {
            Some(db) => db,
            None => return Err(SystemScriptError::TrieDbNotInit.into()),
        };

        let root = CURRENT_HEADER_CELL_ROOT.with(|r| *r.borrow());

        let trie = if root == H256::default() {
            MPTTrie::new(Arc::clone(trie_db))
        } else {
            match MPTTrie::from_root(root, Arc::clone(trie_db)) {
                Ok(m) => m,
                Err(e) => return Err(SystemScriptError::RestoreMpt(e.to_string()).into()),
            }
        };

        Ok(ImageCellStore { trie })
    }

    pub fn update(&mut self, data: image_cell_abi::UpdateCall) -> ProtocolResult<()> {
        for block in data.blocks {
            self.save_cells(block.tx_outputs, block.block_number)?;
            self.mark_cells_consumed(block.tx_inputs, block.block_number)?;
        }

        self.commit()
    }

    pub fn rollback(&mut self, data: image_cell_abi::RollbackCall) -> ProtocolResult<()> {
        for block in data.blocks {
            self.remove_cells(block.tx_outputs)?;
            self.mark_cells_not_consumed(block.tx_inputs)?;
        }

        self.commit()
    }

    fn mark_cells_consumed(
        &mut self,
        inputs: Vec<image_cell_abi::OutPoint>,
        new_block_number: u64,
    ) -> ProtocolResult<()> {
        for input in inputs {
            let key = CellKey::new(input.tx_hash, input.index);

            if let Some(ref mut cell) = self.get_cell(&key)? {
                cell.consumed_number = Some(new_block_number);
                self.insert_cell(&key, cell)?;
            }
        }
        Ok(())
    }

    pub fn remove_cells(&mut self, outputs: Vec<image_cell_abi::OutPoint>) -> ProtocolResult<()> {
        for output in outputs {
            let key = CellKey::new(output.tx_hash, output.index);
            self.remove_cell(&key)?;
        }
        Ok(())
    }

    fn mark_cells_not_consumed(
        &mut self,
        inputs: Vec<image_cell_abi::OutPoint>,
    ) -> ProtocolResult<()> {
        for input in inputs {
            let key = CellKey::new(input.tx_hash, input.index);
            if let Some(ref mut cell) = self.get_cell(&key)? {
                cell.consumed_number = None;
                self.insert_cell(&key, cell)?;
            }
        }
        Ok(())
    }

    pub fn get_cell(&mut self, key: &CellKey) -> ProtocolResult<Option<CellInfo>> {
        let cell = match self.trie.get(&key.encode()) {
            Ok(n) => match n {
                Some(n) => n,
                None => return Ok(None),
            },
            Err(e) => return Err(SystemScriptError::GetCell(e.to_string()).into()),
        };

        Ok(Some(
            rlp::decode(&cell).map_err(SystemScriptError::DecodeCell)?,
        ))
    }

    pub fn save_cells(
        &mut self,
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

            self.insert_cell(&key, &cell_info)?;
        }
        Ok(())
    }

    pub fn insert_cell(&mut self, key: &CellKey, cell: &CellInfo) -> ProtocolResult<()> {
        self.trie
            .insert(&key.encode(), &rlp::encode(cell))
            .map_err(|e| SystemScriptError::InsertCell(e.to_string()).into())
    }

    pub fn remove_cell(&mut self, key: &CellKey) -> ProtocolResult<()> {
        self.trie
            .remove(&key.encode())
            .map_err(|e| SystemScriptError::RemoveCell(e.to_string()).into())
    }

    pub fn commit(&mut self) -> ProtocolResult<()> {
        match self.trie.commit() {
            Ok(new_root) => {
                CURRENT_HEADER_CELL_ROOT.with(|r| *r.borrow_mut() = new_root);
                Ok(())
            }
            Err(e) => Err(SystemScriptError::CommitError(e.to_string()).into()),
        }
    }
}

impl From<&packed::OutPoint> for CellKey {
    fn from(out_point: &packed::OutPoint) -> Self {
        CellKey {
            tx_hash: H256(out_point.tx_hash().unpack().0),
            index:   out_point.index().unpack(),
        }
    }
}

impl CellKey {
    const ENCODED_LEN: usize = 32 + 4;

    pub fn new(tx_hash: [u8; 32], index: u32) -> Self {
        let tx_hash = H256(tx_hash);
        CellKey { tx_hash, index }
    }

    pub fn decode(input: &[u8]) -> ProtocolResult<Self> {
        if input.len() != Self::ENCODED_LEN {
            return Err(SystemScriptError::DataLengthMismatch {
                expect: Self::ENCODED_LEN,
                actual: input.len(),
            }
            .into());
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

impl CellInfo {
    pub fn into_meta(self, out_point: &packed::OutPoint) -> CellMeta {
        CellMeta {
            cell_output:        packed::CellOutput::new_unchecked(self.cell_output),
            out_point:          out_point.clone(),
            transaction_info:   None,
            data_bytes:         self.cell_data.len() as u64,
            mem_cell_data_hash: Some(cell_data_hash(&self.cell_data)),
            mem_cell_data:      Some(self.cell_data),
        }
    }
}

fn cell_data_hash(data: &Bytes) -> packed::Byte32 {
    if !data.is_empty() {
        return ckb_blake2b_256(data).pack();
    }

    packed::Byte32::zero()
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::rand::random;

    #[test]
    fn test_key_codec() {
        for _ in 0..10 {
            let cell_key = CellKey {
                tx_hash: H256::random(),
                index:   random(),
            };

            assert_eq!(CellKey::decode(&cell_key.encode()).unwrap(), cell_key);
        }
    }
}
