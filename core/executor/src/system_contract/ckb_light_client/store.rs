use std::sync::Arc;

use ethers::abi::{AbiDecode, AbiEncode};

use protocol::{types::H256, ProtocolResult};

use crate::system_contract::{
    ckb_light_client::ckb_light_client_abi, error::SystemScriptError, HEADER_CELL_DB,
};
use crate::{adapter::RocksTrieDB, MPTTrie, CURRENT_HEADER_CELL_ROOT};

pub struct CkbLightClientStore {
    pub trie: MPTTrie<RocksTrieDB>,
}

impl CkbLightClientStore {
    pub fn new(root: H256) -> ProtocolResult<Self> {
        let trie_db = match HEADER_CELL_DB.get() {
            Some(db) => db,
            None => return Err(SystemScriptError::TrieDbNotInit.into()),
        };

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
        for header in data.headers {
            self.save_header(&header)?;
        }

        self.commit()
    }

    pub fn rollback(&mut self, data: ckb_light_client_abi::RollbackCall) -> ProtocolResult<()> {
        for block_hash in data.block_hashes {
            self.remove_header(&block_hash)?;
        }

        self.commit()
    }

    pub fn get_header(
        &self,
        block_hash: &[u8],
    ) -> ProtocolResult<Option<ckb_light_client_abi::Header>> {
        let raw = match self.trie.get(block_hash) {
            Ok(n) => match n {
                Some(n) => n,
                None => return Ok(None),
            },
            Err(e) => return Err(SystemScriptError::GetHeader(e.to_string()).into()),
        };

        Ok(Some(
            <ckb_light_client_abi::Header as AbiDecode>::decode(raw)
                .map_err(SystemScriptError::AbiDecode)?,
        ))
    }

    fn save_header(&mut self, header: &ckb_light_client_abi::Header) -> ProtocolResult<()> {
        self.trie
            .insert(&header.block_hash, &header.clone().encode())
            .map_err(|e| SystemScriptError::InsertHeader(e.to_string()).into())
    }

    fn remove_header(&mut self, block_hash: &[u8]) -> ProtocolResult<()> {
        self.trie
            .remove(block_hash)
            .map_err(|e| SystemScriptError::RemoveHeader(e.to_string()).into())
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
