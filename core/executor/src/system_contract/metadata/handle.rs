use protocol::types::{CkbRelatedInfo, Metadata, H160};
use protocol::ProtocolResult;

use crate::system_contract::metadata::MetadataStore;

/// The MetadataHandle is used to expose apis that can be accessed from outside
/// of the system contract.
#[derive(Default)]
pub struct MetadataHandle;

impl MetadataHandle {
    pub fn get_metadata_by_block_number(&self, block_number: u64) -> ProtocolResult<Metadata> {
        let store = MetadataStore::new()?;

        // Should retrieve the first metadata for the genesis block
        if block_number == 0 {
            return store.get_metadata(0);
        }

        let segment = store.get_epoch_segment()?;
        let epoch = segment.get_epoch_number(block_number)?;
        store.get_metadata(epoch)
    }

    pub fn get_metadata_by_epoch(&self, epoch: u64) -> ProtocolResult<Metadata> {
        MetadataStore::new()?.get_metadata(epoch)
    }

    pub fn is_last_block_in_current_epoch(&self, block_number: u64) -> ProtocolResult<bool> {
        let store = MetadataStore::new()?;
        let segment = store.get_epoch_segment()?;
        let is_last_block = segment.is_last_block_in_epoch(block_number);
        Ok(is_last_block)
    }

    pub fn is_validator(&self, block_number: u64, address: H160) -> ProtocolResult<bool> {
        let metadata = self.get_metadata_by_block_number(block_number)?;
        Ok(metadata.verifier_list.iter().any(|v| v.address == address))
    }

    pub fn get_ckb_related_info(&self) -> ProtocolResult<CkbRelatedInfo> {
        MetadataStore::new()?.get_ckb_related_info()
    }
}
