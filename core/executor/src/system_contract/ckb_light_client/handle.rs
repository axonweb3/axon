use ckb_types::packed;
use protocol::types::H256;
use protocol::ProtocolResult;

use crate::system_contract::ckb_light_client::store::CkbLightClientStore;

#[derive(Default)]
pub struct CkbLightClientHandle;

impl CkbLightClientHandle {
    pub fn get_header_by_block_hash(
        &self,
        block_hash: &H256,
    ) -> ProtocolResult<Option<packed::Header>> {
        let store = CkbLightClientStore::new()?;
        store.get_header(block_hash)
    }
}
