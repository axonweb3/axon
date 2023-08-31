use protocol::types::{CkbRelatedInfo, HardforkInfo, Metadata, H160, H256};
use protocol::ProtocolResult;

use crate::system_contract::metadata::MetadataStore;

/// The MetadataHandle is used to expose apis that can be accessed from outside
/// of the system contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataHandle {
    root: H256,
}

impl MetadataHandle {
    pub fn new(root: H256) -> Self {
        MetadataHandle { root }
    }

    pub fn get_metadata_by_block_number(&self, block_number: u64) -> ProtocolResult<Metadata> {
        let store = MetadataStore::new(self.root)?;

        // Should retrieve the first metadata for the genesis block
        if block_number == 0 {
            return store.get_metadata(0);
        }

        let segment = store.get_epoch_segment()?;
        let epoch = segment.get_epoch_number(block_number)?;
        store.get_metadata(epoch)
    }

    pub fn get_metadata_by_epoch(&self, epoch: u64) -> ProtocolResult<Metadata> {
        MetadataStore::new(self.root)?.get_metadata(epoch)
    }

    pub fn is_last_block_in_current_epoch(&self, block_number: u64) -> ProtocolResult<bool> {
        let store = MetadataStore::new(self.root)?;
        let segment = store.get_epoch_segment()?;
        let is_last_block = segment.is_last_block_in_epoch(block_number);
        Ok(is_last_block)
    }

    pub fn is_validator(&self, block_number: u64, address: H160) -> ProtocolResult<bool> {
        let metadata = self.get_metadata_by_block_number(block_number)?;
        Ok(metadata.verifier_list.iter().any(|v| v.address == address))
    }

    pub fn get_ckb_related_info(&self) -> ProtocolResult<CkbRelatedInfo> {
        MetadataStore::new(self.root)?.get_ckb_related_info()
    }

    pub fn hardfork_infos(&self) -> ProtocolResult<HardforkInfo> {
        MetadataStore::new(self.root)?.hardfork_infos()
    }
}
