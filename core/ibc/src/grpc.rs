use std::sync::{RwLock, Arc};

use ibc_proto::ibc::core::client::v1::{
    query_server::{Query as ClientQuery, QueryServer as ClientQueryServer},
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

    msg_server::{Msg as ClientMsg, MsgServer as ClientMsgServer},
    MsgCreateClient, MsgCreateClientResponse, MsgUpdateClient, MsgUpdateClientResponse, MsgUpgradeClient, MsgUpgradeClientResponse, MsgSubmitMisbehaviour, MsgSubmitMisbehaviourResponse,

};
use ibc_proto::ibc::core::client::v1::{
    QueryConsensusStateHeightsRequest, QueryConsensusStateHeightsResponse,
};
use tonic::{Request, Response, Status};

pub trait Store {}
pub trait ProvableStore {}
/// Wraps a store to make it shareable by cloning
#[derive(Clone)]
pub struct SharedStore<S>(Arc<RwLock<S>>);
// static shared_store:SharedStore<i32> = SharedStore::Default();

// pub fn client_service<S: 'static + ProvableStore + Default>() -> ClientQueryServer<IbcClientService<S>> {
//     ClientQueryServer::new(IbcClientService::new(shared_store.clone()))
// }
pub fn client_service() -> ClientQueryServer<IbcClientService> {
    ClientQueryServer::new(IbcClientService::new())
}

pub struct IbcClientService {
// wait for the implementation of ibc/src/adapter.rs
// pub struct IbcClientService<S> {
    //     client_state_store: ProtobufStore<SharedStore<S>, path::ClientStatePath, AnyClientState,
    // Any>, consensus_state_store:
    //     ProtobufStore<SharedStore<S>, path::ClientConsensusStatePath, AnyConsensusState, Any>,
}

// impl<S: Store> IbcClientService<S> {
//     pub fn new(store: SharedStore<S>) -> Self {
impl IbcClientService {
    pub fn new() -> Self {
        Self {
            // client_state_store: TypedStore::new(store.clone()),
            // consensus_state_store: TypedStore::new(store),
        }
    }
}

#[tonic::async_trait]
// impl<S: ProvableStore + 'static> ClientQuery for IbcClientService<S> {
impl ClientQuery for IbcClientService {
    async fn client_state(
        &self,
        _request: Request<QueryClientStateRequest>,
    ) -> Result<Response<QueryClientStateResponse>, Status> {
        // adapter.client_state(_request);
        unimplemented!()
    }

    async fn client_states(
        &self,
        request: Request<QueryClientStatesRequest>,
    ) -> Result<Response<QueryClientStatesResponse>, Status> {
        unimplemented!()
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
        /// SubmitMisbehaviour defines a rpc handler method for MsgSubmitMisbehaviour.
        async fn submit_misbehaviour(
            &self,
            request: tonic::Request<MsgSubmitMisbehaviour>,
        ) -> Result<
            tonic::Response<MsgSubmitMisbehaviourResponse>,
            tonic::Status,
        > {
            unimplemented!()
        }
}