use ibc::{
    core::{
        ics02_client::client_consensus::AnyConsensusState,
        ics02_client::{client_state::AnyClientState, client_type::ClientType},
        ics03_connection::connection::ConnectionEnd,
        ics04_channel::channel::ChannelEnd,
        ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment},
        ics04_channel::packet::{Receipt as IbcReceipt, Sequence},
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
    },
    Height,
};
use protocol::{
    async_trait,
    traits::{Context, IbcAdapter, IbcContext, IbcCrossChainStorage, Storage},
    types::{ConsensusStateWithHeight, Header, Metadata, Path, StoreHeight},
    ProtocolResult,
};
use std::sync::Arc;

macro_rules! blocking_async {
    ($self_: ident, $adapter: ident, $method: ident$ (, $args: expr)*) => {{
        let rt = protocol::tokio::runtime::Handle::current();
        let adapter = Arc::clone(&$self_.$adapter);

        protocol::tokio::task::block_in_place(move || {
            rt.block_on(adapter.$method( $($args,)* )).unwrap()
        })
    }};
}

pub struct DefaultIbcAdapter<S> {
    storage: Arc<S>,
}

impl<S> DefaultIbcAdapter<S>
where
    S: Storage + IbcCrossChainStorage + 'static,
{
    pub async fn new(storage: Arc<S>) -> Self {
        DefaultIbcAdapter { storage }
    }
}

#[async_trait]
impl<S> IbcContext for DefaultIbcAdapter<S>
where
    S: Storage + 'static,
{
    fn get_current_height(&self) -> u64 {
        blocking_async!(self, storage, get_latest_block_header, Context::new()).number
    }
}

// todo : move to storage
#[async_trait]
impl<S> IbcCrossChainStorage for DefaultIbcAdapter<S> {
    fn get_client_type(&self, _client_id: &ClientId) -> ProtocolResult<Option<ClientType>> {
        todo!()
    }

    fn get_client_state(&self, _client_id: &ClientId) -> ProtocolResult<Option<AnyClientState>> {
        todo!()
    }

    fn get_consensus_state(
        &self,
        _client_id: &ClientId,
        _epoch: u64,
        _height: u64,
    ) -> ProtocolResult<Option<AnyConsensusState>> {
        todo!()
    }

    fn get_next_consensus_state(
        &self,
        _client_id: &ClientId,
        _height: Height,
    ) -> ProtocolResult<Option<AnyConsensusState>> {
        todo!()
    }

    fn get_prev_consensus_state(
        &self,
        _client_id: &ClientId,
        _height: Height,
    ) -> ProtocolResult<Option<AnyConsensusState>> {
        todo!()
    }

    fn set_client_type(
        &mut self,
        _client_id: ClientId,
        _client_type: ClientType,
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn set_client_state(
        &mut self,
        _client_id: ClientId,
        _client_state: AnyClientState,
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn set_consensus_state(
        &mut self,
        _client_id: ClientId,
        _height: Height,
        _consensus_state: AnyConsensusState,
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn set_connection_end(
        &mut self,
        _connection_id: ConnectionId,
        _connection_end: ConnectionEnd,
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn set_connection_to_client(
        &mut self,
        _connection_id: ConnectionId,
        _client_id: &ClientId,
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn get_connection_end(&self, _conn_id: &ConnectionId) -> ProtocolResult<Option<ConnectionEnd>> {
        todo!()
    }

    fn set_packet_commitment(
        &mut self,
        _key: (PortId, ChannelId, Sequence),
        _commitment: PacketCommitment,
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn get_packet_commitment(
        &self,
        _key: &(PortId, ChannelId, Sequence),
    ) -> ProtocolResult<Option<PacketCommitment>> {
        todo!()
    }

    fn delete_packet_commitment(
        &mut self,
        _key: (PortId, ChannelId, Sequence),
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn set_packet_receipt(
        &mut self,
        _key: (PortId, ChannelId, Sequence),
        _receipt: IbcReceipt,
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn get_packet_receipt(
        &self,
        _key: &(PortId, ChannelId, Sequence),
    ) -> ProtocolResult<Option<IbcReceipt>> {
        todo!()
    }

    fn set_packet_acknowledgement(
        &mut self,
        _key: (PortId, ChannelId, Sequence),
        _ack_commitment: AcknowledgementCommitment,
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn get_packet_acknowledgement(
        &self,
        _key: &(PortId, ChannelId, Sequence),
    ) -> ProtocolResult<Option<AcknowledgementCommitment>> {
        todo!()
    }

    fn delete_packet_acknowledgement(
        &mut self,
        _key: (PortId, ChannelId, Sequence),
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn set_connection_channels(
        &mut self,
        _conn_id: ConnectionId,
        _port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn set_channel(
        &mut self,
        _port_id: PortId,
        _chan_id: ChannelId,
        _chan_end: ChannelEnd,
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn set_next_sequence_send(
        &mut self,
        _port_id: PortId,
        _chan_id: ChannelId,
        _seq: Sequence,
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn set_next_sequence_recv(
        &mut self,
        _port_id: PortId,
        _chan_id: ChannelId,
        _seq: Sequence,
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn set_next_sequence_ack(
        &mut self,
        _port_id: PortId,
        _chan_id: ChannelId,
        _seq: Sequence,
    ) -> ProtocolResult<()> {
        todo!()
    }

    fn get_channel_end(
        &self,
        _port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<Option<ChannelEnd>> {
        todo!()
    }

    fn get_next_sequence_send(
        &self,
        _port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<Option<Sequence>> {
        todo!()
    }

    fn get_next_sequence_recv(
        &self,
        _port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<Option<Sequence>> {
        todo!()
    }

    fn get_next_sequence_ack(
        &self,
        _port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<Option<Sequence>> {
        todo!()
    }
}

#[async_trait]
impl<S: Send + Sync> IbcAdapter for DefaultIbcAdapter<S> {
    async fn consensus_state_with_height(&self) -> ProtocolResult<ConsensusStateWithHeight> {
        todo!()
    }

    async fn get_metadata(&self, _height: u64) -> ProtocolResult<Metadata> {
        todo!()
    }

    async fn get_header_by_height(&self, _height: u64) -> ProtocolResult<Header> {
        todo!()
    }

    fn get<K, V>(&self, _height: StoreHeight, _path: &K) -> Option<V> {
        todo!()
    }

    fn get_keys(&self, _key_prefix: &Path) -> Vec<Path> {
        todo!()
    }

    fn current_height(&self) -> u64 {
        todo!()
    }
}
