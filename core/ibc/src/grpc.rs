use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use ibc::core::ics02_client::client_consensus::AnyConsensusState;
use ibc::core::ics02_client::client_state::AnyClientState;
use ibc::core::ics02_client::context::{ClientKeeper, ClientReader};
use ibc::core::ics02_client::error::Error;
use ibc::core::ics02_client::events::Attributes;
use ibc::core::ics02_client::handler::ClientResult;
use ibc::core::ics02_client::msgs::create_client::MsgCreateAnyClient;
use ibc::core::ics03_connection::connection::{ConnectionEnd, IdentifiedConnectionEnd};
use ibc::core::ics04_channel::channel::{ChannelEnd, IdentifiedChannelEnd};
use ibc::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::ics04_channel::packet::Sequence;
use ibc::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::ics24_host::{path, Path as IbcPath};
use ibc::core::ics26_routing::context::Ics26Context;
use ibc::events::IbcEvent;
use ibc::handler::{HandlerOutput, HandlerOutputBuilder};

use ibc_proto::ibc::core::client::v1::{
    ConsensusStateWithHeight, QueryConsensusStateHeightsRequest, QueryConsensusStateHeightsResponse,
};
use ibc_proto::ibc::core::{
    channel::v1::{
        query_server::{Query as ChannelQuery, QueryServer as ChannelQueryServer},
        IdentifiedChannel as RawIdentifiedChannel, PacketState, QueryChannelClientStateRequest,
        QueryChannelClientStateResponse, QueryChannelConsensusStateRequest,
        QueryChannelConsensusStateResponse, QueryChannelRequest, QueryChannelResponse,
        QueryChannelsRequest, QueryChannelsResponse, QueryConnectionChannelsRequest,
        QueryConnectionChannelsResponse, QueryNextSequenceReceiveRequest,
        QueryNextSequenceReceiveResponse, QueryPacketAcknowledgementRequest,
        QueryPacketAcknowledgementResponse, QueryPacketAcknowledgementsRequest,
        QueryPacketAcknowledgementsResponse, QueryPacketCommitmentRequest,
        QueryPacketCommitmentResponse, QueryPacketCommitmentsRequest,
        QueryPacketCommitmentsResponse, QueryPacketReceiptRequest, QueryPacketReceiptResponse,
        QueryUnreceivedAcksRequest, QueryUnreceivedAcksResponse, QueryUnreceivedPacketsRequest,
        QueryUnreceivedPacketsResponse,
    },
    client::v1::{
        msg_server::{Msg as ClientMsg, MsgServer as ClientMsgServer},
        query_server::{Query as ClientQuery, QueryServer as ClientQueryServer},
        Height as RawHeight, IdentifiedClientState, MsgCreateClient, MsgCreateClientResponse,
        MsgSubmitMisbehaviour, MsgSubmitMisbehaviourResponse, MsgUpdateClient,
        MsgUpdateClientResponse, MsgUpgradeClient, MsgUpgradeClientResponse,
        QueryClientParamsRequest, QueryClientParamsResponse, QueryClientStateRequest,
        QueryClientStateResponse, QueryClientStatesRequest, QueryClientStatesResponse,
        QueryClientStatusRequest, QueryClientStatusResponse, QueryConsensusStateRequest,
        QueryConsensusStateResponse, QueryConsensusStatesRequest, QueryConsensusStatesResponse,
        QueryUpgradedClientStateRequest, QueryUpgradedClientStateResponse,
        QueryUpgradedConsensusStateRequest, QueryUpgradedConsensusStateResponse,
    },
    connection::v1::{
        query_server::{Query as ConnectionQuery, QueryServer as ConnectionQueryServer},
        IdentifiedConnection as RawIdentifiedConnection, QueryClientConnectionsRequest,
        QueryClientConnectionsResponse, QueryConnectionClientStateRequest,
        QueryConnectionClientStateResponse, QueryConnectionConsensusStateRequest,
        QueryConnectionConsensusStateResponse, QueryConnectionRequest, QueryConnectionResponse,
        QueryConnectionsRequest, QueryConnectionsResponse,
    },
};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use protocol::{
    traits::IbcAdapter,
    types::{Path, StoreHeight as Height},
};

pub const CHAIN_REVISION_NUMBER: u64 = 0;

pub struct GrpcService<Adapter: IbcAdapter, Ctx: Ics26Context> {
    store: Arc<Adapter>,
    addr:  SocketAddr,
    ctx:   Arc<RwLock<Ctx>>,
}

impl<Adapter: IbcAdapter + 'static, Ctx: Ics26Context + Sync + Send + 'static>
    GrpcService<Adapter, Ctx>
{
    pub fn new(adapter: Arc<Adapter>, addr: String, ctx: Arc<RwLock<Ctx>>) -> Self {
        GrpcService {
            store: adapter,
            addr: addr.parse().unwrap(),
            ctx,
        }
    }

    pub async fn run(&self) {
        log::info!("ibc run");
        // [::1] ipv6, equal to 127.0.0.1
        println!("addr {:?}", self.addr);

        let ibc_client_service = self.client_service();
        let ibc_conn_service = self.connection_service();
        let ibc_channel_service = self.channel_service();
        let ibc_client_msg_service = self.client_msg_service();
        Server::builder()
            .add_service(ibc_client_service)
            .add_service(ibc_conn_service)
            .add_service(ibc_channel_service)
            .add_service(ibc_client_msg_service)
            .serve(self.addr)
            .await
            .unwrap();
    }

    pub fn client_service(&self) -> ClientQueryServer<IbcClientService<Adapter>> {
        ClientQueryServer::new(IbcClientService::new(Arc::clone(&self.store)))
    }

    pub fn connection_service(&self) -> ConnectionQueryServer<IbcConnectionService<Adapter>> {
        ConnectionQueryServer::new(IbcConnectionService::new(Arc::clone(&self.store)))
    }

    pub fn channel_service(&self) -> ChannelQueryServer<IbcChannelService<Adapter>> {
        ChannelQueryServer::new(IbcChannelService::new(Arc::clone(&self.store)))
    }

    pub fn client_msg_service(&self) -> ClientMsgServer<IbcClientMsgService<Ctx>> {
        ClientMsgServer::new(IbcClientMsgService::new(Arc::clone(&self.ctx)))
    }
}

pub struct IbcClientService<Adapter: IbcAdapter> {
    adapter: Arc<Adapter>,
}

impl<Adapter: IbcAdapter> IbcClientService<Adapter> {
    pub fn new(adapter: Arc<Adapter>) -> Self {
        Self { adapter }
    }
}

#[tonic::async_trait]
impl<Adapter: IbcAdapter + 'static> ClientQuery for IbcClientService<Adapter> {
    async fn client_state(
        &self,
        _request: Request<QueryClientStateRequest>,
    ) -> Result<Response<QueryClientStateResponse>, Status> {
        unimplemented!()
    }

    async fn client_states(
        &self,
        request: Request<QueryClientStatesRequest>,
    ) -> Result<Response<QueryClientStatesResponse>, Status> {
        log::info!("Got client states request: {:?}", request);

        let path = "clients"
            .to_owned()
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("{:?}", e)))?;

        let client_state_paths = |path: Path| -> Option<path::ClientStatePath> {
            match path.try_into() {
                Ok(IbcPath::ClientState(p)) => Some(p),
                _ => None,
            }
        };

        let identified_client_state = |path: path::ClientStatePath| {
            let client_state: AnyClientState = self.adapter.get(Height::Pending, &path).unwrap();
            IdentifiedClientState {
                client_id:    path.0.to_string(),
                client_state: Some(client_state.into()),
            }
        };

        let keys = self.adapter.get_keys(&path);
        let client_states = keys
            .into_iter()
            .filter_map(client_state_paths)
            .map(identified_client_state)
            .collect();

        Ok(Response::new(QueryClientStatesResponse {
            client_states,
            pagination: None,
        }))
    }

    async fn consensus_state(
        &self,
        _request: Request<QueryConsensusStateRequest>,
    ) -> Result<Response<QueryConsensusStateResponse>, Status> {
        unimplemented!()
    }

    async fn consensus_states(
        &self,
        request: Request<QueryConsensusStatesRequest>,
    ) -> Result<Response<QueryConsensusStatesResponse>, Status> {
        log::info!("Got consensus states request: {:?}", request);

        let path = format!("clients/{}/consensusStates", request.get_ref().client_id)
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("{:?}", e)))?;

        let keys = self.adapter.get_keys(&path);
        let consensus_states = keys
            .into_iter()
            .map(|path| {
                if let Ok(IbcPath::ClientConsensusState(path)) = path.try_into() {
                    let consensus_state: Option<AnyConsensusState> =
                        self.adapter.get(Height::Pending, &path);
                    ConsensusStateWithHeight {
                        height:          Some(RawHeight {
                            revision_number: path.epoch,
                            revision_height: path.height,
                        }),
                        consensus_state: consensus_state.map(|cs| cs.into()),
                    }
                } else {
                    panic!("unexpected path") // safety - store paths are
                                              // assumed to be well-formed
                }
            })
            .collect();

        Ok(Response::new(QueryConsensusStatesResponse {
            consensus_states,
            pagination: None,
        }))
    }

    async fn consensus_state_heights(
        &self,
        _request: Request<QueryConsensusStateHeightsRequest>,
    ) -> Result<Response<QueryConsensusStateHeightsResponse>, Status> {
        unimplemented!()
    }

    async fn client_status(
        &self,
        _request: Request<QueryClientStatusRequest>,
    ) -> Result<Response<QueryClientStatusResponse>, Status> {
        unimplemented!()
    }

    async fn client_params(
        &self,
        _request: Request<QueryClientParamsRequest>,
    ) -> Result<Response<QueryClientParamsResponse>, Status> {
        unimplemented!()
    }

    async fn upgraded_client_state(
        &self,
        _request: Request<QueryUpgradedClientStateRequest>,
    ) -> Result<Response<QueryUpgradedClientStateResponse>, Status> {
        unimplemented!()
    }

    async fn upgraded_consensus_state(
        &self,
        _request: Request<QueryUpgradedConsensusStateRequest>,
    ) -> Result<Response<QueryUpgradedConsensusStateResponse>, Status> {
        unimplemented!()
    }
}

pub struct IbcConnectionService<Adapter: IbcAdapter> {
    connection_end_store: Arc<Adapter>,
    connection_ids_store: Arc<Adapter>,
}

impl<Adapter: IbcAdapter> IbcConnectionService<Adapter> {
    pub fn new(store: Arc<Adapter>) -> Self {
        Self {
            connection_end_store: Arc::clone(&store),
            connection_ids_store: Arc::clone(&store),
        }
    }
}

#[tonic::async_trait]
impl<Adapter: IbcAdapter + 'static> ConnectionQuery for IbcConnectionService<Adapter> {
    async fn connection(
        &self,
        request: Request<QueryConnectionRequest>,
    ) -> Result<Response<QueryConnectionResponse>, Status> {
        let conn_id = ConnectionId::from_str(&request.get_ref().connection_id)
            .map_err(|_| Status::invalid_argument("invalid connection id"))?;
        let conn: Option<ConnectionEnd> = self
            .connection_end_store
            .get(Height::Pending, &path::ConnectionsPath(conn_id));
        Ok(Response::new(QueryConnectionResponse {
            connection:   conn.map(|c| c.into()),
            proof:        vec![],
            proof_height: None,
        }))
    }

    async fn connections(
        &self,
        _request: Request<QueryConnectionsRequest>,
    ) -> Result<Response<QueryConnectionsResponse>, Status> {
        let connection_path_prefix: Path = String::from("connections")
            .try_into()
            .expect("'connections' expected to be a valid Path");

        let connection_paths = self.connection_end_store.get_keys(&connection_path_prefix);

        let identified_connections: Vec<RawIdentifiedConnection> = connection_paths
            .into_iter()
            .map(|path| match path.try_into() {
                Ok(IbcPath::Connections(connections_path)) => {
                    let connection_end = self
                        .connection_end_store
                        .get(Height::Pending, &connections_path)
                        .unwrap();
                    IdentifiedConnectionEnd::new(connections_path.0, connection_end).into()
                }
                _ => panic!("unexpected path"),
            })
            .collect();

        Ok(Response::new(QueryConnectionsResponse {
            connections: identified_connections,
            pagination:  None,
            height:      None,
        }))
    }

    async fn client_connections(
        &self,
        request: Request<QueryClientConnectionsRequest>,
    ) -> Result<Response<QueryClientConnectionsResponse>, Status> {
        let client_id = request
            .get_ref()
            .client_id
            .parse()
            .map_err(|e| Status::invalid_argument(format!("{}", e)))?;
        let path = path::ClientConnectionsPath(client_id);
        let connection_ids: Vec<ConnectionId> = self
            .connection_ids_store
            .get(Height::Pending, &path)
            .unwrap_or_default();
        let connection_paths = connection_ids
            .into_iter()
            .map(|conn_id| conn_id.to_string())
            .collect();

        Ok(Response::new(QueryClientConnectionsResponse {
            connection_paths,
            // Note: proofs aren't being used by hermes currently
            proof: vec![],
            proof_height: None,
        }))
    }

    async fn connection_client_state(
        &self,
        _request: Request<QueryConnectionClientStateRequest>,
    ) -> Result<Response<QueryConnectionClientStateResponse>, Status> {
        todo!()
    }

    async fn connection_consensus_state(
        &self,
        _request: Request<QueryConnectionConsensusStateRequest>,
    ) -> Result<Response<QueryConnectionConsensusStateResponse>, Status> {
        todo!()
    }
}

pub struct IbcChannelService<Adapter: IbcAdapter> {
    channel_end_store:       Arc<Adapter>,
    packet_commitment_store: Arc<Adapter>,
    packet_ack_store:        Arc<Adapter>,
    packet_receipt_store:    Arc<Adapter>,
}

impl<Adapter: IbcAdapter> IbcChannelService<Adapter> {
    pub fn new(store: Arc<Adapter>) -> Self {
        Self {
            channel_end_store:       Arc::clone(&store),
            packet_commitment_store: Arc::clone(&store),
            packet_ack_store:        Arc::clone(&store),
            packet_receipt_store:    Arc::clone(&store),
        }
    }
}

#[tonic::async_trait]
impl<Adapter: IbcAdapter + 'static> ChannelQuery for IbcChannelService<Adapter> {
    async fn channel(
        &self,
        request: Request<QueryChannelRequest>,
    ) -> Result<Response<QueryChannelResponse>, Status> {
        let request = request.into_inner();
        let port_id = PortId::from_str(&request.port_id)
            .map_err(|_| Status::invalid_argument("invalid port id"))?;
        let channel_id = ChannelId::from_str(&request.channel_id)
            .map_err(|_| Status::invalid_argument("invalid channel id"))?;

        let channel = self
            .channel_end_store
            .get(Height::Pending, &path::ChannelEndsPath(port_id, channel_id))
            .map(|channel_end: ChannelEnd| channel_end.into());

        Ok(Response::new(QueryChannelResponse {
            channel,
            proof: vec![],
            proof_height: None,
        }))
    }

    /// Channels queries all the IBC channels of a chain.
    async fn channels(
        &self,
        _request: Request<QueryChannelsRequest>,
    ) -> Result<Response<QueryChannelsResponse>, Status> {
        let channel_path_prefix: Path = String::from("channelEnds/ports")
            .try_into()
            .expect("'channelEnds/ports' expected to be a valid Path");

        let channel_paths = self.channel_end_store.get_keys(&channel_path_prefix);
        let identified_channels: Vec<RawIdentifiedChannel> = channel_paths
            .into_iter()
            .map(|path| match path.try_into() {
                Ok(IbcPath::ChannelEnds(channels_path)) => {
                    let channel_end = self
                        .channel_end_store
                        .get(Height::Pending, &channels_path)
                        .expect("channel path returned by get_keys() had no associated channel");
                    IdentifiedChannelEnd::new(channels_path.0, channels_path.1, channel_end).into()
                }
                _ => panic!("unexpected path"),
            })
            .collect();

        Ok(Response::new(QueryChannelsResponse {
            channels:   identified_channels,
            pagination: None,
            height:     Some(RawHeight {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.channel_end_store.current_height(),
            }),
        }))
    }

    /// ConnectionChannels queries all the channels associated with a connection
    /// end.
    async fn connection_channels(
        &self,
        request: Request<QueryConnectionChannelsRequest>,
    ) -> Result<Response<QueryConnectionChannelsResponse>, Status> {
        let conn_id = ConnectionId::from_str(&request.get_ref().connection)
            .map_err(|_| Status::invalid_argument("invalid connection id"))?;

        let path = "channelEnds"
            .to_owned()
            .try_into()
            .expect("'commitments/ports' expected to be a valid Path");

        let keys = self.channel_end_store.get_keys(&path);
        let channels = keys
            .into_iter()
            .filter_map(|path| {
                if let Ok(IbcPath::ChannelEnds(path)) = path.try_into() {
                    let channel_end: ChannelEnd =
                        self.channel_end_store.get(Height::Pending, &path)?;
                    if channel_end.connection_hops.first() == Some(&conn_id) {
                        return Some(IdentifiedChannelEnd::new(path.0, path.1, channel_end).into());
                    }
                }

                None
            })
            .collect();

        Ok(Response::new(QueryConnectionChannelsResponse {
            channels,
            pagination: None,
            height: Some(RawHeight {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.channel_end_store.current_height(),
            }),
        }))
    }

    /// ChannelClientState queries for the client state for the channel
    /// associated with the provided channel identifiers.
    async fn channel_client_state(
        &self,
        _request: Request<QueryChannelClientStateRequest>,
    ) -> Result<Response<QueryChannelClientStateResponse>, Status> {
        todo!()
    }

    /// ChannelConsensusState queries for the consensus state for the channel
    /// associated with the provided channel identifiers.
    async fn channel_consensus_state(
        &self,
        _request: Request<QueryChannelConsensusStateRequest>,
    ) -> Result<Response<QueryChannelConsensusStateResponse>, Status> {
        todo!()
    }

    /// PacketCommitment queries a stored packet commitment hash.
    async fn packet_commitment(
        &self,
        _request: Request<QueryPacketCommitmentRequest>,
    ) -> Result<Response<QueryPacketCommitmentResponse>, Status> {
        todo!()
    }

    /// PacketCommitments returns all the packet commitments hashes associated
    /// with a channel.
    async fn packet_commitments(
        &self,
        request: Request<QueryPacketCommitmentsRequest>,
    ) -> Result<Response<QueryPacketCommitmentsResponse>, Status> {
        let request = request.into_inner();
        let port_id = PortId::from_str(&request.port_id)
            .map_err(|_| Status::invalid_argument("invalid port id"))?;
        let channel_id = ChannelId::from_str(&request.channel_id)
            .map_err(|_| Status::invalid_argument("invalid channel id"))?;

        let commitment_paths = {
            let prefix: Path = String::from("commitments/ports")
                .try_into()
                .expect("'commitments/ports' expected to be a valid Path");
            self.packet_commitment_store.get_keys(&prefix)
        };

        let matching_commitment_paths = |path: Path| -> Option<path::CommitmentsPath> {
            match path.try_into() {
                Ok(IbcPath::Commitments(p))
                    if p.port_id == port_id && p.channel_id == channel_id =>
                {
                    Some(p)
                }
                _ => None,
            }
        };

        let packet_state = |path: path::CommitmentsPath| -> Option<PacketState> {
            let commitment: PacketCommitment = self
                .packet_commitment_store
                .get(Height::Pending, &path)
                .unwrap();
            let data = commitment.into_vec();
            (!data.is_empty()).then(|| PacketState {
                port_id: path.port_id.to_string(),
                channel_id: path.channel_id.to_string(),
                sequence: path.sequence.into(),
                data,
            })
        };

        let packet_states: Vec<PacketState> = commitment_paths
            .into_iter()
            .filter_map(matching_commitment_paths)
            .filter_map(packet_state)
            .collect();

        Ok(Response::new(QueryPacketCommitmentsResponse {
            commitments: packet_states,
            pagination:  None,
            height:      Some(RawHeight {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.packet_commitment_store.current_height(),
            }),
        }))
    }

    /// PacketReceipt queries if a given packet sequence has been received on
    /// the queried chain
    async fn packet_receipt(
        &self,
        _request: Request<QueryPacketReceiptRequest>,
    ) -> Result<Response<QueryPacketReceiptResponse>, Status> {
        todo!()
    }

    /// PacketAcknowledgement queries a stored packet acknowledgement hash.
    async fn packet_acknowledgement(
        &self,
        _request: Request<QueryPacketAcknowledgementRequest>,
    ) -> Result<Response<QueryPacketAcknowledgementResponse>, Status> {
        todo!()
    }

    /// PacketAcknowledgements returns all the packet acknowledgements
    /// associated with a channel.
    async fn packet_acknowledgements(
        &self,
        request: Request<QueryPacketAcknowledgementsRequest>,
    ) -> Result<Response<QueryPacketAcknowledgementsResponse>, Status> {
        let request = request.into_inner();
        let port_id = PortId::from_str(&request.port_id)
            .map_err(|_| Status::invalid_argument("invalid port id"))?;
        let channel_id = ChannelId::from_str(&request.channel_id)
            .map_err(|_| Status::invalid_argument("invalid channel id"))?;

        let ack_paths = {
            let prefix: Path = String::from("acks/ports")
                .try_into()
                .expect("'acks/ports' expected to be a valid Path");
            self.packet_ack_store.get_keys(&prefix)
        };

        let matching_ack_paths = |path: Path| -> Option<path::AcksPath> {
            match path.try_into() {
                Ok(IbcPath::Acks(p)) if p.port_id == port_id && p.channel_id == channel_id => {
                    Some(p)
                }
                _ => None,
            }
        };

        let packet_state = |path: path::AcksPath| -> Option<PacketState> {
            let commitment: AcknowledgementCommitment =
                self.packet_ack_store.get(Height::Pending, &path).unwrap();
            let data = commitment.into_vec();
            (!data.is_empty()).then(|| PacketState {
                port_id: path.port_id.to_string(),
                channel_id: path.channel_id.to_string(),
                sequence: path.sequence.into(),
                data,
            })
        };

        let packet_states: Vec<PacketState> = ack_paths
            .into_iter()
            .filter_map(matching_ack_paths)
            .filter_map(packet_state)
            .collect();

        Ok(Response::new(QueryPacketAcknowledgementsResponse {
            acknowledgements: packet_states,
            pagination:       None,
            height:           Some(RawHeight {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.packet_ack_store.current_height(),
            }),
        }))
    }

    /// UnreceivedPackets returns all the unreceived IBC packets associated with
    /// a channel and sequences.
    ///
    /// QUESTION. Currently only works for unordered channels; ordered channels
    /// don't use receipts. However, ibc-go does it this way. Investigate if
    /// this query only ever makes sense on unordered channels.
    async fn unreceived_packets(
        &self,
        request: Request<QueryUnreceivedPacketsRequest>,
    ) -> Result<Response<QueryUnreceivedPacketsResponse>, Status> {
        let request = request.into_inner();
        let port_id = PortId::from_str(&request.port_id)
            .map_err(|_| Status::invalid_argument("invalid port id"))?;
        let channel_id = ChannelId::from_str(&request.channel_id)
            .map_err(|_| Status::invalid_argument("invalid channel id"))?;
        let sequences_to_check: Vec<u64> = request.packet_commitment_sequences;

        let unreceived_sequences: Vec<u64> = sequences_to_check
            .into_iter()
            .filter(|seq| {
                let receipts_path = path::ReceiptsPath {
                    port_id:    port_id.clone(),
                    channel_id: channel_id.clone(),
                    sequence:   Sequence::from(*seq),
                };
                let packet_receipt: Option<()> = self
                    .packet_receipt_store
                    .get(Height::Pending, &receipts_path);
                packet_receipt.is_none()
            })
            .collect();

        Ok(Response::new(QueryUnreceivedPacketsResponse {
            sequences: unreceived_sequences,
            height:    Some(RawHeight {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.packet_receipt_store.current_height(),
            }),
        }))
    }

    /// UnreceivedAcks returns all the unreceived IBC acknowledgements
    /// associated with a channel and sequences.
    async fn unreceived_acks(
        &self,
        request: Request<QueryUnreceivedAcksRequest>,
    ) -> Result<Response<QueryUnreceivedAcksResponse>, Status> {
        let request = request.into_inner();
        let port_id = PortId::from_str(&request.port_id)
            .map_err(|_| Status::invalid_argument("invalid port id"))?;
        let channel_id = ChannelId::from_str(&request.channel_id)
            .map_err(|_| Status::invalid_argument("invalid channel id"))?;
        let sequences_to_check: Vec<u64> = request.packet_ack_sequences;

        let unreceived_sequences: Vec<u64> = sequences_to_check
            .into_iter()
            .filter(|seq| {
                // To check if we received an acknowledgement, we check if we still have the
                // sent packet commitment (upon receiving an ack, the sent
                // packet commitment is deleted).
                let commitments_path = path::CommitmentsPath {
                    port_id:    port_id.clone(),
                    channel_id: channel_id.clone(),
                    sequence:   Sequence::from(*seq),
                };

                let packet_commitment: Option<PacketCommitment> = self
                    .packet_commitment_store
                    .get(Height::Pending, &commitments_path);
                packet_commitment.is_some()
            })
            .collect();

        Ok(Response::new(QueryUnreceivedAcksResponse {
            sequences: unreceived_sequences,
            height:    Some(RawHeight {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.packet_commitment_store.current_height(),
            }),
        }))
    }

    /// NextSequenceReceive returns the next receive sequence for a given
    /// channel.
    async fn next_sequence_receive(
        &self,
        _request: Request<QueryNextSequenceReceiveRequest>,
    ) -> Result<Response<QueryNextSequenceReceiveResponse>, Status> {
        todo!()
    }
}

pub struct IbcClientMsgService<Ctx: ClientReader + ClientKeeper> {
    ctx: Arc<RwLock<Ctx>>,
}

impl<Ctx: ClientReader + ClientKeeper> IbcClientMsgService<Ctx> {
    pub fn new(ctx: Arc<RwLock<Ctx>>) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl<Ctx: ClientReader + ClientKeeper + Sync + Send + 'static> ClientMsg
    for IbcClientMsgService<Ctx>
{
    /// CreateClient defines a rpc handler method for MsgCreateClient.
    async fn create_client(
        &self,
        request: tonic::Request<MsgCreateClient>,
    ) -> Result<tonic::Response<MsgCreateClientResponse>, tonic::Status> {
        let raw = request.get_ref();
        let msg = MsgCreateAnyClient::try_from(raw.clone()).unwrap();

        let mut output: HandlerOutputBuilder<ClientResult> = HandlerOutput::builder();

        // Construct this client's identifier
        let mut ctx = self.ctx.write().unwrap();
        let id_counter = ctx.client_counter().unwrap();
        let client_id = ClientId::new(msg.client_state.client_type(), id_counter)
            .map_err(|e| {
                Error::client_identifier_constructor(msg.client_state.client_type(), id_counter, e)
            })
            .unwrap();

        output.log(format!(
            "success: generated new client identifier: {}",
            client_id
        ));
        use ibc::core::ics02_client::handler::create_client::Result;
        let result = ClientResult::Create(Result {
            client_id:        client_id.clone(),
            client_type:      msg.client_state.client_type(),
            client_state:     msg.client_state.clone(),
            consensus_state:  msg.consensus_state,
            processed_time:   ctx.host_timestamp(),
            processed_height: ctx.host_height(),
        });

        let event_attributes = Attributes {
            client_id,
            ..Default::default()
        };
        output.emit(IbcEvent::CreateClient(event_attributes.into()));

        // Apply the result to the context (host chain store).
        ctx.store_client_result(result)
            .map_err(|_v| tonic::Status::invalid_argument("store_client_result"))?;

        let res = tonic::Response::<MsgCreateClientResponse>::new(MsgCreateClientResponse {});

        Ok(res)
    }

    /// UpdateClient defines a rpc handler method for MsgUpdateClient.
    async fn update_client(
        &self,
        _request: tonic::Request<MsgUpdateClient>,
    ) -> Result<tonic::Response<MsgUpdateClientResponse>, tonic::Status> {
        unimplemented!()
    }

    /// UpgradeClient defines a rpc handler method for MsgUpgradeClient.
    async fn upgrade_client(
        &self,
        _request: tonic::Request<MsgUpgradeClient>,
    ) -> Result<tonic::Response<MsgUpgradeClientResponse>, tonic::Status> {
        unimplemented!()
    }

    /// SubmitMisbehaviour defines a rpc handler method for
    /// MsgSubmitMisbehaviour.
    async fn submit_misbehaviour(
        &self,
        _request: tonic::Request<MsgSubmitMisbehaviour>,
    ) -> Result<tonic::Response<MsgSubmitMisbehaviourResponse>, tonic::Status> {
        unimplemented!()
    }
}
