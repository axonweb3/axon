use std::str::FromStr;
use std::sync::{Arc, RwLock};

use ibc::core::ics24_host::{path, Path as IbcPath};
use ibc_proto::ibc::core::client::v1::{
    msg_server::{Msg as ClientMsg, MsgServer as ClientMsgServer},
    query_server::{Query as ClientQuery, QueryServer as ClientQueryServer},
    IdentifiedClientState,
    MsgCreateClient,
    MsgCreateClientResponse,
    MsgSubmitMisbehaviour,
    MsgSubmitMisbehaviourResponse,
    MsgUpdateClient,
    MsgUpdateClientResponse,
    MsgUpgradeClient,
    MsgUpgradeClientResponse,
    // ConsensusStateWithHeight, Height as RawHeight, IdentifiedClientState,
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
};
use ibc_proto::ibc::core::client::v1::{
    QueryConsensusStateHeightsRequest, QueryConsensusStateHeightsResponse,
};
use tonic::{Request, Response, Status};

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

        let keys = self.adapter.get_keys(&path.try_into().unwrap());
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
        unimplemented!()
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
