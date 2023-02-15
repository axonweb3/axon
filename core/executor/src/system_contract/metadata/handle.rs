use protocol::types::{Metadata, H160};
use protocol::ProtocolResult;

use crate::system_contract::metadata::MetadataStore;

#[derive(Default)]
pub struct MetadataHandle;

impl MetadataHandle {
    pub fn get_metadata_by_block_number(&self, block_number: u64) -> ProtocolResult<Metadata> {
        let store = MetadataStore::new()?;
        let segment = store.get_epoch_segment()?;
        let epoch = segment.get_epoch_number(block_number)?;
        store.get_metadata(epoch)
    }

    pub fn get_metadata_by_epoch(&self, epoch: u64) -> ProtocolResult<Metadata> {
        MetadataStore::new()?.get_metadata(epoch)
    }

    pub fn is_validator(&self, block_number: u64, address: H160) -> ProtocolResult<bool> {
        let metadata = self.get_metadata_by_block_number(block_number)?;
        Ok(metadata.verifier_list.iter().any(|v| v.address == address))
    }
}
