use cosmos_ibc::core::ics02_client::client_consensus::AnyConsensusState;
use cosmos_ibc::core::ics02_client::client_state::AnyClientState;
use cosmos_ibc::core::ics02_client::client_type::ClientType;
use cosmos_ibc::core::ics03_connection::connection::ConnectionEnd;
use cosmos_ibc::core::ics04_channel::channel::ChannelEnd;
use cosmos_ibc::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use cosmos_ibc::core::ics04_channel::packet::{Receipt, Sequence};
use cosmos_ibc::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use cosmos_ibc::core::ics24_host::path::{
    AcksPath, ChannelEndsPath, ClientConnectionsPath, ClientConsensusStatePath, ClientStatePath,
    CommitmentsPath, ConnectionsPath, ReceiptsPath,
};
use creep::Context;

use crate::types::{Header, Metadata, Path, StoreHeight};
use crate::{async_trait, ProtocolResult};

#[async_trait]
pub trait IbcGrpcAdapter {
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

    fn get_paths_by_prefix(&self, key_prefix: &Path) -> ProtocolResult<Vec<Path>>;
}

#[async_trait]
pub trait IbcAdapter: IbcGrpcAdapter + Send + Sync {
    async fn get_metadata(&self, height: u64) -> ProtocolResult<Metadata>;

    async fn get_header_by_height(&self, height: u64) -> ProtocolResult<Header>;

    fn get_client_type(
        &self,
        ctx: Context,
        client_id: &ClientId,
    ) -> ProtocolResult<Option<ClientType>>;

    fn get_current_client_state(
        &self,
        ctx: Context,
        client_id: &ClientId,
    ) -> ProtocolResult<Option<AnyClientState>>;

    fn get_current_consensus_state(
        &self,
        ctx: Context,
        client_id: &ClientId,
        epoch: u64,
        height: u64,
    ) -> ProtocolResult<Option<AnyConsensusState>>;

    fn get_next_consensus_state(
        &self,
        ctx: Context,
        client_id: &ClientId,
        height: cosmos_ibc::Height,
    ) -> ProtocolResult<Option<AnyConsensusState>>;

    fn get_prev_consensus_state(
        &self,
        ctx: Context,
        client_id: &ClientId,
        height: cosmos_ibc::Height,
    ) -> ProtocolResult<Option<AnyConsensusState>>;

    fn get_connection_end_by_id(
        &self,
        ctx: Context,
        conn_id: &ConnectionId,
    ) -> ProtocolResult<Option<ConnectionEnd>>;

    fn get_channel_end_by_id(
        &self,
        ctx: Context,
        port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<Option<ChannelEnd>>;

    fn get_next_sequence_send(
        &self,
        ctx: Context,
        port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<Option<Sequence>>;

    fn get_next_sequence_recv(
        &self,
        ctx: Context,
        port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<Option<Sequence>>;

    fn get_next_sequence_ack(
        &self,
        ctx: Context,
        port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<Option<Sequence>>;

    fn get_current_packet_commitment(
        &self,
        ctx: Context,
        key: &(PortId, ChannelId, Sequence),
    ) -> ProtocolResult<Option<PacketCommitment>>;

    fn get_packet_receipt(
        &self,
        ctx: Context,
        key: &(PortId, ChannelId, Sequence),
    ) -> ProtocolResult<Option<Receipt>>;

    fn get_packet_acknowledgement(
        &self,
        ctx: Context,
        key: &(PortId, ChannelId, Sequence),
    ) -> ProtocolResult<Option<AcknowledgementCommitment>>;

    fn set_client_type(
        &mut self,
        ctx: Context,
        client_id: ClientId,
        client_type: ClientType,
    ) -> ProtocolResult<()>;

    fn set_client_state(
        &self,
        ctx: Context,
        client_id: ClientId,
        client_state: AnyClientState,
    ) -> ProtocolResult<()>;

    fn set_consensus_state(
        &self,
        ctx: Context,
        client_id: ClientId,
        height: cosmos_ibc::Height,
        consensus_state: AnyConsensusState,
    ) -> ProtocolResult<()>;

    fn set_connection_end(
        &self,
        ctx: Context,
        connection_id: ConnectionId,
        connection_end: ConnectionEnd,
    ) -> ProtocolResult<()>;

    fn set_connection_to_client(
        &self,
        ctx: Context,
        connection_id: ConnectionId,
        client_id: ClientId,
    ) -> ProtocolResult<()>;

    fn set_packet_commitment(
        &self,
        ctx: Context,
        key: (PortId, ChannelId, Sequence),
        commitment: PacketCommitment,
    ) -> ProtocolResult<()>;

    fn set_packet_receipt(
        &self,
        ctx: Context,
        key: (PortId, ChannelId, Sequence),
        receipt: Receipt,
    ) -> ProtocolResult<()>;

    fn set_packet_acknowledgement(
        &self,
        ctx: Context,
        key: (PortId, ChannelId, Sequence),
        ack_commitment: AcknowledgementCommitment,
    ) -> ProtocolResult<()>;

    fn set_channel(
        &self,
        ctx: Context,
        port_id: PortId,
        chan_id: ChannelId,
        channel_end: ChannelEnd,
    ) -> ProtocolResult<()>;

    fn set_next_sequence_send(
        &self,
        ctx: Context,
        port_id: PortId,
        chan_id: ChannelId,
        seq: Sequence,
    ) -> ProtocolResult<()>;

    fn set_next_sequence_recv(
        &self,
        ctx: Context,
        port_id: PortId,
        chan_id: ChannelId,
        seq: Sequence,
    ) -> ProtocolResult<()>;

    fn set_next_sequence_ack(
        &self,
        ctx: Context,
        port_id: PortId,
        chan_id: ChannelId,
        seq: Sequence,
    ) -> ProtocolResult<()>;

    fn remove_packet_commitment(
        &self,
        ctx: Context,
        key: (PortId, ChannelId, Sequence),
    ) -> ProtocolResult<()>;

    fn current_height(&self) -> u64;
}
