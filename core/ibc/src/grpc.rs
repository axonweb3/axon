use std::str::FromStr;
use std::sync::{Arc, RwLock};

use ibc::core::ics03_connection::connection::IdentifiedConnectionEnd;
use ibc::core::ics04_channel::channel::IdentifiedChannelEnd;
use ibc::core::ics04_channel::packet::Sequence;
use ibc::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use ibc::core::ics24_host::path::ClientConnectionsPath;
use ibc::core::ics24_host::{path, Path as IbcPath};
use ibc_proto::ibc::core::client::v1::{
    ConsensusStateWithHeight, QueryConsensusStateHeightsRequest, QueryConsensusStateHeightsResponse,
};
use ibc_proto::ibc::core::{
    channel::v1::{
        query_server::{Query as ChannelQuery, QueryServer as ChannelQueryServer},
        Channel as RawChannelEnd, IdentifiedChannel as RawIdentifiedChannel, PacketState,
        QueryChannelClientStateRequest, QueryChannelClientStateResponse,
        QueryChannelConsensusStateRequest, QueryChannelConsensusStateResponse, QueryChannelRequest,
        QueryChannelResponse, QueryChannelsRequest, QueryChannelsResponse,
        QueryConnectionChannelsRequest, QueryConnectionChannelsResponse,
        QueryNextSequenceReceiveRequest, QueryNextSequenceReceiveResponse,
        QueryPacketAcknowledgementRequest, QueryPacketAcknowledgementResponse,
        QueryPacketAcknowledgementsRequest, QueryPacketAcknowledgementsResponse,
        QueryPacketCommitmentRequest, QueryPacketCommitmentResponse, QueryPacketCommitmentsRequest,
        QueryPacketCommitmentsResponse, QueryPacketReceiptRequest, QueryPacketReceiptResponse,
        QueryUnreceivedAcksRequest, QueryUnreceivedAcksResponse, QueryUnreceivedPacketsRequest,
        QueryUnreceivedPacketsResponse,
    },
    client::v1::{
        msg_server::{Msg as ClientMsg, MsgServer as ClientMsgServer},
        query_server::{Query as ClientQuery, QueryServer as ClientQueryServer},
        // ConsensusStateWithHeight,
        Height as RawHeight,
        IdentifiedClientState,
        MsgCreateClient,
        MsgCreateClientResponse,
        MsgSubmitMisbehaviour,
        MsgSubmitMisbehaviourResponse,
        MsgUpdateClient,
        MsgUpdateClientResponse,
        MsgUpgradeClient,
        MsgUpgradeClientResponse,
        QueryClientParamsRequest,
        QueryClientParamsResponse,
        QueryClientStateRequest,
        QueryClientStateResponse,
        QueryClientStatesRequest,
        QueryClientStatesResponse,
        QueryClientStatusRequest,
        QueryClientStatusResponse,
        QueryConsensusStateRequest,
        QueryConsensusStateResponse,
        QueryConsensusStatesRequest,
        QueryConsensusStatesResponse,
        QueryUpgradedClientStateRequest,
        QueryUpgradedClientStateResponse,
        QueryUpgradedConsensusStateRequest,
        QueryUpgradedConsensusStateResponse,
    },
    connection::v1::{
        query_server::{Query as ConnectionQuery, QueryServer as ConnectionQueryServer},
        ConnectionEnd as RawConnectionEnd, IdentifiedConnection as RawIdentifiedConnection,
        QueryClientConnectionsRequest, QueryClientConnectionsResponse,
        QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
        QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
        QueryConnectionRequest, QueryConnectionResponse, QueryConnectionsRequest,
        QueryConnectionsResponse,
    },
};
use tonic::{Request, Response, Status};

use crate::adapter::{
    AcknowledgementCommitmentSchema, ChannelEndSchema, ClientConsensusStateSchema,
    ConnectionEndSchema, ConnectionIdsSchema, PacketCommitmentSchema, ReceiptSchema,
};
use crate::store::{Height, CHAIN_REVISION_NUMBER};
use crate::{
    adapter::{ClientStateSchema, IbcAdapter},
    store::Path,
};

impl TryFrom<Path> for IbcPath {
    type Error = path::PathError;

    fn try_from(path: Path) -> Result<Self, Self::Error> {
        Self::from_str(path.to_string().as_str())
    }
}

impl From<IbcPath> for Path {
    fn from(ibc_path: IbcPath) -> Self {
        Self::try_from(ibc_path.to_string()).unwrap() // safety - `IbcPath`s are
                                                      // correct-by-construction
    }
}

macro_rules! impl_into_path_for {
    ($($path:ty),+) => {
        $(impl From<$path> for Path {
            fn from(ibc_path: $path) -> Self {
                Self::try_from(ibc_path.to_string()).unwrap() // safety - `IbcPath`s are correct-by-construction
            }
        })+
    };
}

impl_into_path_for!(
    path::ClientTypePath,
    path::ClientStatePath,
    path::ClientConsensusStatePath,
    path::ConnectionsPath,
    path::ClientConnectionsPath,
    path::ChannelEndsPath,
    path::SeqSendsPath,
    path::SeqRecvsPath,
    path::SeqAcksPath,
    path::CommitmentsPath,
    path::ReceiptsPath,
    path::AcksPath
);

pub struct IbcClientService<S> {
    //     client_state_store: ProtobufStore<SharedStore<S>, path::ClientStatePath, AnyClientState,
    // Any>, consensus_state_store:
    //     ProtobufStore<SharedStore<S>, path::ClientConsensusStatePath, AnyConsensusState, Any>,
    adapter: Arc<S>,
}

// impl<S: Store> IbcClientService<S> {
//     pub fn new(store: SharedStore<S>) -> Self {
impl<S: IbcAdapter> IbcClientService<S> {
    pub fn new(store: Arc<S>) -> Self {
        Self {
            adapter: store,
            // client_state_store: TypedStore::new(store.clone()),
            // consensus_state_store: TypedStore::new(store),
        }
    }
}

#[tonic::async_trait]
// impl<S: ProvableStore + 'static> ClientQuery for IbcClientService<S> {
impl<S: IbcAdapter + 'static + Send + Sync> ClientQuery for IbcClientService<S> {
    async fn client_state(
        &self,
        _request: Request<QueryClientStateRequest>,
    ) -> Result<Response<QueryClientStateResponse>, Status> {
        // self.adapter.client_state(_request);
        unimplemented!()
    }

    async fn client_states(
        &self,
        request: Request<QueryClientStatesRequest>,
    ) -> Result<Response<QueryClientStatesResponse>, Status> {
        // trace!("Got client states request: {:?}", request);

        let path: Path = "clients"
            .to_owned()
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("{}", e)))?;

        let client_state_paths = |path: Path| -> Option<path::ClientStatePath> {
            match path.try_into() {
                Ok(IbcPath::ClientState(p)) => Some(p),
                _ => None,
            }
        };

        let identified_client_state = |path: path::ClientStatePath| {
            let client_state = self.adapter.get::<ClientStateSchema>(&path).unwrap();
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
            pagination: None, // TODO(hu55a1n1): add pagination support
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
        let path: Path = format!("clients/{}/consensusStates", request.get_ref().client_id)
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("{}", e)))?;

        let keys = self.adapter.get_keys(&path);
        let consensus_states = keys
            .into_iter()
            .map(|path| {
                if let Ok(IbcPath::ClientConsensusState(path)) = path.try_into() {
                    let consensus_state = Some(
                        self.adapter
                            .get::<ClientConsensusStateSchema>(&path)
                            .unwrap(),
                    );
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
            pagination: None, // TODO(hu55a1n1): add pagination support
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

pub struct IbcConnectionService<S> {
    adapter: Arc<S>,
}

impl<S: IbcAdapter> IbcConnectionService<S> {
    pub fn new(store: Arc<S>) -> Self {
        Self { adapter: store }
    }
}

#[tonic::async_trait]
impl<S: IbcAdapter + 'static + Send + Sync> ConnectionQuery for IbcConnectionService<S> {
    async fn connection(
        &self,
        request: Request<QueryConnectionRequest>,
    ) -> Result<Response<QueryConnectionResponse>, Status> {
        let conn_id = ConnectionId::from_str(&request.get_ref().connection_id)
            .map_err(|_| Status::invalid_argument("invalid connection id"))?;
        let conn = Some(
            self.adapter
                .get::<ConnectionEndSchema>(&path::ConnectionsPath(conn_id))
                .unwrap(),
        );
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

        let connection_paths = self.adapter.get_keys(&connection_path_prefix);

        let identified_connections: Vec<RawIdentifiedConnection> = connection_paths
            .into_iter()
            .map(|path| match path.try_into() {
                Ok(IbcPath::Connections(connections_path)) => {
                    let connection_end = self
                        .adapter
                        .get::<ConnectionEndSchema>(&connections_path)
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
        let connection_ids = self
            .adapter
            .get::<ConnectionIdsSchema>(&path)
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

pub struct IbcChannelService<S> {
    adapter: Arc<S>,
}

impl<S: IbcAdapter> IbcChannelService<S> {
    pub fn new(store: Arc<S>) -> Self {
        Self { adapter: store }
    }
}

#[tonic::async_trait]
impl<S: IbcAdapter + 'static + Send + Sync> ChannelQuery for IbcChannelService<S> {
    async fn channel(
        &self,
        request: Request<QueryChannelRequest>,
    ) -> Result<Response<QueryChannelResponse>, Status> {
        let request = request.into_inner();
        let port_id = PortId::from_str(&request.port_id)
            .map_err(|_| Status::invalid_argument("invalid port id"))?;
        let channel_id = ChannelId::from_str(&request.channel_id)
            .map_err(|_| Status::invalid_argument("invalid channel id"))?;

        let channel = Some(
            self.adapter
                .get::<ChannelEndSchema>(&path::ChannelEndsPath(port_id, channel_id))
                .unwrap(),
        )
        .map(|channel_end| channel_end.into());

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

        let channel_paths = self.adapter.get_keys(&channel_path_prefix);
        let identified_channels: Vec<RawIdentifiedChannel> = channel_paths
            .into_iter()
            .map(|path| match path.try_into() {
                Ok(IbcPath::ChannelEnds(channels_path)) => {
                    let channel_end = self
                        .adapter
                        .get::<ChannelEndSchema>(&channels_path)
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
                revision_height: self.adapter.get_current_height(),
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

        let keys = self.adapter.get_keys(&path);
        let channels = keys
            .into_iter()
            .filter_map(|path| {
                if let Ok(IbcPath::ChannelEnds(path)) = path.try_into() {
                    let channel_end = self.adapter.get::<ChannelEndSchema>(&path).unwrap();
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
                revision_height: self.adapter.get_current_height(),
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
            self.adapter.get_keys(&prefix)
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
            let commitment = self.adapter.get::<PacketCommitmentSchema>(&path).unwrap();
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
                revision_height: self.adapter.get_current_height(),
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
            self.adapter.get_keys(&prefix)
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
            let commitment = self
                .adapter
                .get::<AcknowledgementCommitmentSchema>(&path)
                .unwrap();
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
                revision_height: self.adapter.get_current_height(),
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
                Some(self.adapter.get::<ReceiptSchema>(&receipts_path).unwrap()).is_none()
            })
            .collect();

        Ok(Response::new(QueryUnreceivedPacketsResponse {
            sequences: unreceived_sequences,
            height:    Some(RawHeight {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.adapter.get_current_height(),
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

                Some(
                    self.adapter
                        .get::<PacketCommitmentSchema>(&commitments_path),
                )
                .is_some()
            })
            .collect();

        Ok(Response::new(QueryUnreceivedAcksResponse {
            sequences: unreceived_sequences,
            height:    Some(RawHeight {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.adapter.get_current_height(),
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

pub fn client_msg_service() -> ClientMsgServer<IbcClientMsgService> {
    ClientMsgServer::new(IbcClientMsgService::new())
}

pub struct IbcClientMsgService {
    // client_state_store: ProtobufStore<SharedStore<S>, path::ClientStatePath, AnyClientState,
    // Any>, consensus_state_store:
    //     ProtobufStore<SharedStore<S>, path::ClientConsensusStatePath, AnyConsensusState, Any>,
}

impl IbcClientMsgService {
    pub fn new() -> Self {
        Self {
            // client_state_store: TypedStore::new(store.clone()),
            // consensus_state_store: TypedStore::new(store),
        }
    }
}
#[tonic::async_trait]
impl ClientMsg for IbcClientMsgService {
    /// CreateClient defines a rpc handler method for MsgCreateClient.
    async fn create_client(
        &self,
        request: tonic::Request<MsgCreateClient>,
    ) -> Result<tonic::Response<MsgCreateClientResponse>, tonic::Status> {
        unimplemented!()
    }

    /// UpdateClient defines a rpc handler method for MsgUpdateClient.
    async fn update_client(
        &self,
        request: tonic::Request<MsgUpdateClient>,
    ) -> Result<tonic::Response<MsgUpdateClientResponse>, tonic::Status> {
        unimplemented!()
    }

    /// UpgradeClient defines a rpc handler method for MsgUpgradeClient.
    async fn upgrade_client(
        &self,
        request: tonic::Request<MsgUpgradeClient>,
    ) -> Result<tonic::Response<MsgUpgradeClientResponse>, tonic::Status> {
        unimplemented!()
    }

    /// SubmitMisbehaviour defines a rpc handler method for
    /// MsgSubmitMisbehaviour.
    async fn submit_misbehaviour(
        &self,
        request: tonic::Request<MsgSubmitMisbehaviour>,
    ) -> Result<tonic::Response<MsgSubmitMisbehaviourResponse>, tonic::Status> {
        unimplemented!()
    }
}
