use cosmos_ibc::core::ics02_client::client_consensus::AnyConsensusState;
use cosmos_ibc::core::ics02_client::client_state::AnyClientState;
use cosmos_ibc::core::ics03_connection::connection::ConnectionEnd;
use cosmos_ibc::core::ics04_channel::channel::ChannelEnd;
use cosmos_ibc::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use cosmos_ibc::core::ics24_host::identifier::ConnectionId;
use cosmos_ibc::core::ics24_host::path::{
    AcksPath, ChannelEndsPath, ClientConnectionsPath, ClientConsensusStatePath, ClientStatePath,
    CommitmentsPath, ConnectionsPath, ReceiptsPath,
};

use crate::traits::IbcCrossChainStorage;
use crate::types::{
    ConsensusStateWithHeight, Header, Metadata, Path, StoreHeight, StoreHeight as Height,
};
use crate::{async_trait, ProtocolResult};

#[async_trait]
pub trait IbcAdapter: Send + Sync {
    async fn consensus_state_with_height(&self) -> ProtocolResult<ConsensusStateWithHeight>;

    async fn get_metadata(&self, height: u64) -> ProtocolResult<Metadata>;

    async fn get_header_by_height(&self, height: u64) -> ProtocolResult<Header>;

    async fn get_client_state(
        &self,
        height: StoreHeight,
        path: &ClientStatePath,
    ) -> ProtocolResult<Option<AnyClientState>>;

    async fn get_consensus_state(
        &self,
        height: StoreHeight,
        path: &ClientConsensusStatePath,
    ) -> ProtocolResult<Option<AnyConsensusState>>;

    async fn get_connection_end(
        &self,
        height: StoreHeight,
        path: &ConnectionsPath,
    ) -> ProtocolResult<Option<ConnectionEnd>>;

    async fn get_connection_ids(
        &self,
        height: StoreHeight,
        path: &ClientConnectionsPath,
    ) -> ProtocolResult<Vec<ConnectionId>>;

    async fn get_acknowledgement_commitment(
        &self,
        height: StoreHeight,
        path: &AcksPath,
    ) -> ProtocolResult<Option<AcknowledgementCommitment>>;

    async fn get_channel_end(
        &self,
        height: StoreHeight,
        path: &ChannelEndsPath,
    ) -> ProtocolResult<Option<ChannelEnd>>;

    fn get_opt(&self, height: StoreHeight, path: &ReceiptsPath) -> ProtocolResult<Option<()>>;

    fn get_packet_commitment(
        &self,
        height: StoreHeight,
        path: &CommitmentsPath,
    ) -> ProtocolResult<Option<PacketCommitment>>;

    fn get<K, V>(&self, height: Height, path: &K) -> Option<V>;

    fn get_paths_by_prefix(&self, key_prefix: &Path) -> ProtocolResult<Vec<Path>>;

    fn current_height(&self) -> u64;
}

pub trait IbcContext: IbcCrossChainStorage {
    fn get_current_height(&self) -> u64;
}
