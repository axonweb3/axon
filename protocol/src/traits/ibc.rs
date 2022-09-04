use crate::traits::IbcCrossChainStorage;
use crate::types::{ConsensusStateWithHeight, Header, Metadata};
use crate::{async_trait, traits::Context, ProtocolResult};

#[async_trait]
pub trait IbcAdapter: Send + Sync {
    async fn consensus_state_with_height(
        &self,
        ctx: Context,
    ) -> ProtocolResult<ConsensusStateWithHeight>;

    async fn get_metadata(&self, ctx: Context, height: u64) -> ProtocolResult<Metadata>;

    async fn get_header_by_height(&self, ctx: Context, height: u64) -> ProtocolResult<Header>;
}

pub trait IbcContext: IbcCrossChainStorage {
    // fn get_current_height(&self) -> u64;
}
