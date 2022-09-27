use core_storage::StorageError;
use ibc::{
    core::{
        ics02_client::client_consensus::AnyConsensusState,
        ics02_client::{client_state::AnyClientState, client_type::ClientType},
        ics03_connection::connection::ConnectionEnd,
        ics04_channel::channel::ChannelEnd,
        ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment},
        ics04_channel::packet::{Receipt as IbcReceipt, Sequence},
        ics24_host::{
            identifier::{ChannelId, ClientId, ConnectionId, PortId},
            path::{
                AcksPath, ChannelEndsPath, ClientConnectionsPath, ClientConsensusStatePath,
                ClientStatePath, CommitmentsPath, ConnectionsPath, ReceiptsPath,
            },
        },
    },
    Height,
};
use protocol::{
    async_trait,
    traits::{Context, IbcAdapter, IbcCrossChainStorage, IbcGrpcAdapter, MetadataControl, Storage},
    types::{Header, Metadata, Path, StoreHeight},
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

pub struct DefaultIbcAdapter<S, MT> {
    storage:  Arc<S>,
    metadata: Arc<MT>,
}

impl<S, MT> DefaultIbcAdapter<S, MT>
where
    S: Storage + IbcCrossChainStorage + 'static,
    MT: MetadataControl + 'static,
{
    pub async fn new(storage: Arc<S>, metadata: Arc<MT>) -> Self {
        DefaultIbcAdapter { storage, metadata }
    }
}

#[async_trait]
impl<S, MT> IbcGrpcAdapter for DefaultIbcAdapter<S, MT>
where
    S: Storage + IbcCrossChainStorage + 'static,
    MT: MetadataControl + 'static,
{
    async fn get_client_state(
        &self,
        _height: StoreHeight,
        path: &ClientStatePath,
    ) -> ProtocolResult<Option<AnyClientState>> {
        self.storage.get_client_state(&path.0)
    }

    async fn get_consensus_state(
        &self,
        _height: StoreHeight,
        path: &ClientConsensusStatePath,
    ) -> ProtocolResult<Option<AnyConsensusState>> {
        self.storage
            .get_consensus_state(&path.client_id, path.epoch, path.height)
    }

    async fn get_connection_end(
        &self,
        _height: StoreHeight,
        path: &ConnectionsPath,
    ) -> ProtocolResult<Option<ConnectionEnd>> {
        self.storage.get_connection_end(&path.0)
    }

    async fn get_connection_ids(
        &self,
        _height: StoreHeight,
        path: &ClientConnectionsPath,
    ) -> ProtocolResult<Vec<ConnectionId>> {
        let ret = self
            .storage
            .get_connection_to_client(&path.0)?
            .unwrap_or_default();
        Ok(ret)
    }

    async fn get_acknowledgement_commitment(
        &self,
        _height: StoreHeight,
        path: &AcksPath,
    ) -> ProtocolResult<Option<AcknowledgementCommitment>> {
        self.storage.get_packet_acknowledgement(&(
            path.port_id.clone(),
            path.channel_id.clone(),
            path.sequence,
        ))
    }

    async fn get_channel_end(
        &self,
        _height: StoreHeight,
        path: &ChannelEndsPath,
    ) -> ProtocolResult<Option<ChannelEnd>> {
        self.storage
            .get_channel_end(&(path.0.clone(), path.1.clone()))
    }

    fn get_opt(&self, _height: StoreHeight, path: &ReceiptsPath) -> ProtocolResult<Option<()>> {
        self.storage.get_packet_receipt(&(
            path.port_id.clone(),
            path.channel_id.clone(),
            path.sequence,
        ))?;
        Ok(Some(()))
    }

    fn get_packet_commitment(
        &self,
        _height: StoreHeight,
        path: &CommitmentsPath,
    ) -> ProtocolResult<Option<PacketCommitment>> {
        self.storage.get_packet_commitment(&(
            path.port_id.clone(),
            path.channel_id.clone(),
            path.sequence,
        ))
    }

    fn get_paths_by_prefix(&self, key_prefix: &Path) -> ProtocolResult<Vec<Path>> {
        self.storage.get_keys_by_prefix(key_prefix)
    }
}

#[async_trait]
impl<S, MT> IbcAdapter for DefaultIbcAdapter<S, MT>
where
    S: Storage + IbcCrossChainStorage + 'static,
    MT: MetadataControl + 'static,
{
    async fn get_metadata(&self, height: u64) -> ProtocolResult<Metadata> {
        let header = self.get_header_by_height(height).await.unwrap();
        self.metadata.get_metadata(Context::new(), &header)
    }

    async fn get_header_by_height(&self, height: u64) -> ProtocolResult<Header> {
        match self.storage.get_block_header(Context::new(), height).await {
            Ok(Some(header)) => Ok(header),
            Ok(None) => Err(StorageError::GetNone((height).to_string()).into()),
            Err(_) => Err(StorageError::GetNone("DB error".to_string()).into()),
        }
    }

    fn get_client_type(
        &self,
        _ctx: Context,
        client_id: &ClientId,
    ) -> ProtocolResult<Option<ClientType>> {
        self.storage.get_client_type(client_id)
    }

    fn get_current_client_state(
        &self,
        _ctx: Context,
        client_id: &ClientId,
    ) -> ProtocolResult<Option<AnyClientState>> {
        self.storage.get_client_state(client_id)
    }

    fn get_current_consensus_state(
        &self,
        _ctx: Context,
        client_id: &ClientId,
        epoch: u64,
        height: u64,
    ) -> ProtocolResult<Option<AnyConsensusState>> {
        self.storage.get_consensus_state(client_id, epoch, height)
    }

    fn get_next_consensus_state(
        &self,
        _ctx: Context,
        client_id: &ClientId,
        height: Height,
    ) -> ProtocolResult<Option<AnyConsensusState>> {
        self.storage.get_next_consensus_state(client_id, height)
    }

    fn get_prev_consensus_state(
        &self,
        _ctx: Context,
        client_id: &ClientId,
        height: Height,
    ) -> ProtocolResult<Option<AnyConsensusState>> {
        self.storage.get_prev_consensus_state(client_id, height)
    }

    fn get_connection_end_by_id(
        &self,
        _ctx: Context,
        conn_id: &ConnectionId,
    ) -> ProtocolResult<Option<ConnectionEnd>> {
        self.storage.get_connection_end(conn_id)
    }

    fn get_channel_end_by_id(
        &self,
        _ctx: Context,
        port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<Option<ChannelEnd>> {
        self.storage.get_channel_end(port_channel_id)
    }

    fn get_next_sequence_send(
        &self,
        _ctx: Context,
        port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<Option<Sequence>> {
        self.storage.get_next_sequence_send(port_channel_id)
    }

    fn get_next_sequence_recv(
        &self,
        _ctx: Context,
        port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<Option<Sequence>> {
        self.storage.get_next_sequence_recv(port_channel_id)
    }

    fn get_next_sequence_ack(
        &self,
        _ctx: Context,
        port_channel_id: &(PortId, ChannelId),
    ) -> ProtocolResult<Option<Sequence>> {
        self.storage.get_next_sequence_ack(port_channel_id)
    }

    fn get_current_packet_commitment(
        &self,
        _ctx: Context,
        key: &(PortId, ChannelId, Sequence),
    ) -> ProtocolResult<Option<PacketCommitment>> {
        self.storage.get_packet_commitment(key)
    }

    fn get_packet_receipt(
        &self,
        _ctx: Context,
        key: &(PortId, ChannelId, Sequence),
    ) -> ProtocolResult<Option<IbcReceipt>> {
        self.storage.get_packet_receipt(key)
    }

    fn get_packet_acknowledgement(
        &self,
        _ctx: Context,
        key: &(PortId, ChannelId, Sequence),
    ) -> ProtocolResult<Option<AcknowledgementCommitment>> {
        self.storage.get_packet_acknowledgement(key)
    }

    fn set_client_type(
        &self,
        _ctx: Context,
        client_id: ClientId,
        client_type: ClientType,
    ) -> ProtocolResult<()> {
        self.storage.set_client_type(client_id, client_type)
    }

    fn set_client_state(
        &self,
        _ctx: Context,
        client_id: ClientId,
        client_state: AnyClientState,
    ) -> ProtocolResult<()> {
        self.storage.set_client_state(client_id, client_state)
    }

    fn set_consensus_state(
        &self,
        _ctx: Context,
        client_id: ClientId,
        height: Height,
        consensus_state: AnyConsensusState,
    ) -> ProtocolResult<()> {
        self.storage
            .set_consensus_state(client_id, height, consensus_state)
    }

    fn set_connection_end(
        &self,
        _ctx: Context,
        connection_id: ConnectionId,
        connection_end: ConnectionEnd,
    ) -> ProtocolResult<()> {
        self.storage
            .set_connection_end(connection_id, connection_end)
    }

    fn set_connection_to_client(
        &self,
        _ctx: Context,
        connection_id: ConnectionId,
        client_id: ClientId,
    ) -> ProtocolResult<()> {
        self.storage
            .set_connection_to_client(connection_id, &client_id)
    }

    fn set_packet_commitment(
        &self,
        _ctx: Context,
        key: (PortId, ChannelId, Sequence),
        commitment: PacketCommitment,
    ) -> ProtocolResult<()> {
        self.storage.set_packet_commitment(key, commitment)
    }

    fn set_packet_receipt(
        &self,
        _ctx: Context,
        key: (PortId, ChannelId, Sequence),
        receipt: IbcReceipt,
    ) -> ProtocolResult<()> {
        self.storage.set_packet_receipt(key, receipt)
    }

    fn set_packet_acknowledgement(
        &self,
        _ctx: Context,
        key: (PortId, ChannelId, Sequence),
        ack_commitment: AcknowledgementCommitment,
    ) -> ProtocolResult<()> {
        self.storage.set_packet_acknowledgement(key, ack_commitment)
    }

    fn set_channel(
        &self,
        _ctx: Context,
        port_id: PortId,
        chan_id: ChannelId,
        channel_end: ChannelEnd,
    ) -> ProtocolResult<()> {
        self.storage.set_channel(port_id, chan_id, channel_end)
    }

    fn set_next_sequence_send(
        &self,
        _ctx: Context,
        port_id: PortId,
        chan_id: ChannelId,
        seq: Sequence,
    ) -> ProtocolResult<()> {
        self.storage.set_next_sequence_send(port_id, chan_id, seq)
    }

    fn set_next_sequence_recv(
        &self,
        _ctx: Context,
        port_id: PortId,
        chan_id: ChannelId,
        seq: Sequence,
    ) -> ProtocolResult<()> {
        self.storage.set_next_sequence_recv(port_id, chan_id, seq)
    }

    fn set_next_sequence_ack(
        &self,
        _ctx: Context,
        port_id: PortId,
        chan_id: ChannelId,
        seq: Sequence,
    ) -> ProtocolResult<()> {
        self.storage.set_next_sequence_ack(port_id, chan_id, seq)
    }

    fn remove_packet_commitment(
        &self,
        _ctx: Context,
        key: (PortId, ChannelId, Sequence),
    ) -> ProtocolResult<()> {
        self.storage.delete_packet_commitment(key)
    }

    fn current_height(&self) -> u64 {
        blocking_async!(self, storage, get_latest_block_header, Context::new()).number
    }
}
