use std::sync::Arc;

use ckb_types::packed;
use ckb_types::prelude::{Builder, Entity, Pack};
use protocol::types::{MerkleRoot, H256};
use protocol::ProtocolResult;

use crate::system_contract::error::SystemScriptError;
use crate::system_contract::trie_db::RocksTrieDB;
use crate::MPTTrie;

use crate::system_contract::ckb_light_client::{ckb_light_client_abi, CURRENT_CELL_ROOT, TRIE_DB};

pub struct CkbLightClientStore {
    pub trie: MPTTrie<RocksTrieDB>,
}

impl CkbLightClientStore {
    pub fn new() -> ProtocolResult<Self> {
        let trie_db = match TRIE_DB.get() {
            Some(db) => db,
            None => return Err(SystemScriptError::TrieDbNotInit.into()),
        };

        let root = **CURRENT_CELL_ROOT.load();

        let trie = if root == H256::default() {
            let mut m = MPTTrie::new(Arc::clone(trie_db));

            m
        } else {
            match MPTTrie::from_root(root, Arc::clone(trie_db)) {
                Ok(m) => m,
                Err(e) => return Err(SystemScriptError::RestoreMpt(e.to_string()).into()),
            }
        };

        Ok(CkbLightClientStore { trie })
    }

    pub fn update(&self, data: ckb_light_client_abi::UpdateCall) -> ProtocolResult<MerkleRoot> {
        self.save_header(&data.header)?;

        self.commit()
    }

    pub fn rollback(&self, data: ckb_light_client_abi::RollbackCall) -> ProtocolResult<MerkleRoot> {
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

    fn save_header(&self, header: &ckb_light_client_abi::Header) -> ProtocolResult<()> {
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

    fn remove_header(&self, key: &H256) -> ProtocolResult<()> {
        self.trie
            .remove(&key.0)
            .map_err(|e| SystemScriptError::RemoveHeader(e.to_string()).into())
    }

    pub fn commit(&self) -> ProtocolResult<MerkleRoot> {
        self.trie
            .commit()
            .map_err(|e| SystemScriptError::CommitError(e.to_string()).into())
    }
}
