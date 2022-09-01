use crate::traits::IbcCrossChainStorage;
use crate::types::{ConsensusStateWithHeight, Header, Metadata};
use crate::{async_trait, ProtocolResult};

#[async_trait]
pub trait IbcAdapter: Send + Sync {
    async fn consensus_state_with_height(&self) -> ProtocolResult<ConsensusStateWithHeight>;

    async fn get_metadata(&self, height: u64) -> ProtocolResult<Metadata>;

    async fn get_header_by_height(&self, height: u64) -> ProtocolResult<Header>;
}

pub trait IbcContext: IbcCrossChainStorage {
    fn get_current_height(&self) -> u64;
}
