use std::sync::Arc;

use crate::MPTTrie;
use ckb_types::packed;
use ckb_types::prelude::{Builder, Entity, Pack};
use protocol::types::H256;
use protocol::ProtocolResult;

use crate::system_contract::{
    ckb_light_client::ckb_light_client_abi, error::SystemScriptError, trie_db::RocksTrieDB,
    CURRENT_HEADER_CELL_ROOT, HEADER_CELL_DB,
};

pub struct CkbLightClientStore {
    pub trie: MPTTrie<RocksTrieDB>,
}

impl CkbLightClientStore {
    pub fn new() -> ProtocolResult<Self> {
        let trie_db = match HEADER_CELL_DB.get() {
            Some(db) => db,
            None => return Err(SystemScriptError::TrieDbNotInit.into()),
        };

        let root = **CURRENT_HEADER_CELL_ROOT.load();

        let trie = if root == H256::default() {
            MPTTrie::new(Arc::clone(trie_db))
        } else {
            match MPTTrie::from_root(root, Arc::clone(trie_db)) {
                Ok(m) => m,
                Err(e) => return Err(SystemScriptError::RestoreMpt(e.to_string()).into()),
            }
        };

        Ok(CkbLightClientStore { trie })
    }

    pub fn update(&mut self, data: ckb_light_client_abi::UpdateCall) -> ProtocolResult<()> {
        self.save_header(&data.header)?;

        self.commit()
    }

    pub fn rollback(&mut self, data: ckb_light_client_abi::RollbackCall) -> ProtocolResult<()> {
        self.remove_header(&H256(data.block_hash))?;

        self.commit()
    }

    pub fn get_header(&self, block_hash: &H256) -> ProtocolResult<Option<packed::Header>> {
        let header = match self.trie.get(&block_hash.0) {
            Ok(n) => match n {
                Some(n) => n,
                None => return Ok(None),
            },
            Err(e) => return Err(SystemScriptError::GetHeader(e.to_string()).into()),
        };

        Ok(Some(
            packed::Header::from_slice(&header).map_err(SystemScriptError::MoleculeVerification)?,
        ))
    }

    fn save_header(&mut self, header: &ckb_light_client_abi::Header) -> ProtocolResult<()> {
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

        self.trie
            .insert(&H256(header.block_hash).0, &packed_header.as_bytes())
            .map_err(|e| SystemScriptError::InsertHeader(e.to_string()).into())
    }

    fn remove_header(&mut self, key: &H256) -> ProtocolResult<()> {
        self.trie
            .remove(&key.0)
            .map_err(|e| SystemScriptError::RemoveHeader(e.to_string()).into())
    }

    pub fn commit(&mut self) -> ProtocolResult<()> {
        match self.trie.commit() {
            Ok(new_root) => {
                CURRENT_HEADER_CELL_ROOT.swap(Arc::new(new_root));
                Ok(())
            }
            Err(e) => Err(SystemScriptError::CommitError(e.to_string()).into()),
        }
    }
}
