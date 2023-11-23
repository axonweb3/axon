use std::sync::Arc;

use ethers::abi::{AbiDecode, AbiEncode};

use protocol::trie::Trie as _;
use protocol::{codec::hex_encode, types::H256, ProtocolResult};

use crate::system_contract::{
    ckb_light_client::ckb_light_client_abi, error::SystemScriptError, HEADER_CELL_DB,
};
use crate::{adapter::RocksTrieDB, MPTTrie, CURRENT_HEADER_CELL_ROOT};

/// The CKB light client store does not follow the storage layout of EVM smart
/// contract. It use MPT called HeaderCell MPT with the following layout:
/// | key                   | value                    |
/// | --------------------- | ------------------------ |
/// | CKB Header Hash       | `CkbHeader.encode()`     |
/// | ...                   | ...                      |
/// | CellOutpoint.encode() | `CellInfo.encode()`      |
/// | ...                   | ...                      |
///
/// All these data are stored in a the `c10` column family of RocksDB, and the
/// root of the HeaderCell MPT is stored in the storage MPT of the CKB light
/// client and image cell contract account as follow:
///
/// **CKB light client Account**
/// | address | `0xFfFfFFFfFFfFFFFfFFFFffFfFFfffFFFfffffF02`|
/// | nonce   | `0x0`                                       |
/// | balance | `0x0`                                       |
/// | storage | `storage_root`                              |
///
/// **CKB light client Storage MPT**
/// | HEADER_CELL_ROOT_KEY | HeaderCell MPT root |
///
/// **Image cell Account**
/// | address | `0xffffffffFfFFffffFFFfffFfFFfFffFffFFFff03`|
/// | nonce   | `0x0`                                       |
/// | balance | `0x0`                                       |
/// | storage | `storage_root`                              |
///
/// **Image cell Storage MPT**
/// | HEADER_CELL_ROOT_KEY | HeaderCell MPT root |
///
/// **Notice**: The `storage_root` of the CKB light client and image cell
/// contract Account are same, so once the HeaderCell MPT has been changed, the
/// `storage_root` of the CKB light client and image cell both need to be
/// updated.
pub struct CkbLightClientStore {
    pub trie: MPTTrie<RocksTrieDB>,
}

impl CkbLightClientStore {
    pub fn new(root: H256) -> ProtocolResult<Self> {
        let trie_db = {
            let lock = HEADER_CELL_DB.read();
            match lock.clone() {
                Some(db) => db,
                None => return Err(SystemScriptError::TrieDbNotInit.into()),
            }
        };

        let trie =
            if root == H256::default() {
                MPTTrie::new(Arc::clone(&trie_db))
            } else {
                match MPTTrie::from_root(root, Arc::clone(&trie_db)) {
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
            .insert(header.block_hash.to_vec(), header.clone().encode().to_vec())
            .map_err(|e| SystemScriptError::InsertHeader(e.to_string()).into())
    }

    fn remove_header(&mut self, block_hash: &[u8]) -> ProtocolResult<()> {
        self.trie
            .remove(block_hash)
            .map_err(|e| SystemScriptError::RemoveHeader(e.to_string()))
            .and_then(|removed| {
                if removed {
                    Ok(())
                } else {
                    let content = format!("remove header {} failed", hex_encode(block_hash));
                    Err(SystemScriptError::RemoveHeader(content))
                }
            })
            .map_err(Into::into)
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
