mod adapter;
mod client;
mod codec;
mod error;
mod grpc;
mod transfer;

use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use protocol::ProtocolResult;
use tonic::transport::Server;

use crate::grpc::{client_service, client_msg_service};

use ibc::clients::ics07_tendermint::consensus_state::ConsensusState;
use ibc::timestamp::Timestamp;
use ibc::{
    core::{
        ics02_client::client_consensus::AnyConsensusState,
        ics02_client::context::ClientReader,
        ics02_client::error::Error as ClientError,
        ics02_client::{
            client_state::AnyClientState, client_type::ClientType, context::ClientKeeper,
        },
        ics03_connection::connection::ConnectionEnd,
        ics03_connection::context::{ConnectionKeeper, ConnectionReader},
        ics03_connection::error::Error as ConnectionError,
        ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment},
        ics04_channel::context::ChannelReader,
        ics04_channel::error::Error as ChannelError,
        ics04_channel::packet::{Receipt, Sequence},
        ics04_channel::{channel::ChannelEnd, context::ChannelKeeper},
        ics05_port::context::PortReader,
        ics05_port::error::Error as PortError,
        ics23_commitment::commitment::CommitmentPrefix,
        ics24_host::{
            identifier::{ChannelId, ClientId, ConnectionId, PortId},
            path::{
                AcksPath, ChannelEndsPath, ClientConnectionsPath, ClientConsensusStatePath,
                ClientStatePath, ClientTypePath, CommitmentsPath, ConnectionsPath, ReceiptsPath,
                SeqAcksPath, SeqRecvsPath, SeqSendsPath,
            },
        },
        ics26_routing::context::{Ics26Context, Module, ModuleId, Router},
    },
    Height,
};

use adapter::{
    AcknowledgementCommitmentSchema, ChannelEndSchema, ClientConsensusStateSchema,
    ClientStateSchema, ClientTypeSchema, ConnectionEndSchema, ConnectionIdsSchema, IbcAdapter,
    PacketCommitmentSchema, ReceiptSchema, SeqAcksSchema, SeqRecvsSchema, SeqSendsSchema, CrossChainCodec, StoreSchema,
};
use protocol::types::Hasher;

pub struct IbcImpl<Adapter: IbcAdapter, Router> {
    adapter:                  Arc<Adapter>,
    router:                   Router,
    client_counter:           u64,
    channel_counter:          u64,
    conn_counter:             u64,
    port_to_module_map:       BTreeMap<PortId, ModuleId>,
    client_processed_times:   HashMap<(ClientId, Height), Timestamp>,
    client_processed_heights: HashMap<(ClientId, Height), Height>,
    consensus_states:         HashMap<u64, ConsensusState>,
}

impl<Adapter: IbcAdapter, Router> IbcImpl<Adapter, Router> {
    pub fn new() -> Self {
        IbcImpl {
            adapter: todo!(),
            router: todo!(),
            client_counter: todo!(),
            channel_counter: todo!(),
            conn_counter: todo!(),
            port_to_module_map: todo!(),
            client_processed_times: todo!(),
            client_processed_heights: todo!(),
            consensus_states: todo!(),
        }
    }

pub async fn run(&self) {
    log::info!("ibc run");

    // [::1] ipv6, equal to 127.0.0.1
    let addr = "[::1]:50051".parse().unwrap();

    let ibc_client_service = client_service();
    let ibc_client_msg_service = client_msg_service();

    let _grpc = Server::builder()
    .add_service(ibc_client_service)
    .add_service(ibc_client_msg_service)
    // .add_service(ibc_conn_service)
    // .add_service(ibc_channel_service)
    .serve(addr).await.unwrap();
}
}

impl<Adapter: IbcAdapter, Router> ClientReader for IbcImpl<Adapter, Router> {
    fn client_type(&self, client_id: &ClientId) -> Result<ClientType, ClientError> {
        self.adapter
            .get::<ClientTypeSchema>(&ClientTypePath(client_id.clone()))
            .map_err(|_| ClientError::implementation_specific())
        // .ok_or_else(ClientError::implementation_specific)
    }

    fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, ClientError> {
        self.adapter
            .get::<ClientStateSchema>(&ClientStatePath(client_id.clone()))
            .map_err(|_| ClientError::implementation_specific())
    }

    fn consensus_state(
        &self,
        client_id: &ClientId,
        height: ibc::Height,
    ) -> Result<AnyConsensusState, ClientError> {
        let path = ClientConsensusStatePath {
            client_id: client_id.clone(),
            epoch:     height.revision_number(),
            height:    height.revision_height(),
        };
        self.adapter
            .get::<ClientConsensusStateSchema>(&path)
            .map_err(|_| ClientError::implementation_specific())
    }

    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: ibc::Height,
    ) -> Result<Option<AnyConsensusState>, ClientError> {
        let keys: &[ClientConsensusStatePath] =
            self.adapter.get_all_keys::<ClientConsensusStateSchema>();
        let found_path = keys.iter().find(|path| {
            &path.client_id == client_id && height > Height::new(path.epoch, path.height).unwrap()
        });
        if let Some(path) = found_path {
            let consensus_state = self
                .adapter
                .get::<ClientConsensusStateSchema>(path)
                .map_err(|_| ClientError::consensus_state_not_found(client_id.clone(), height))?;
            Ok(Some(consensus_state))
        } else {
            Ok(None)
        }
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: ibc::Height,
    ) -> Result<Option<AnyConsensusState>, ClientError> {
        let keys: &[ClientConsensusStatePath] =
            self.adapter.get_all_keys::<ClientConsensusStateSchema>();
        let pos = keys.iter().position(|path| {
            &path.client_id == client_id && height >= Height::new(path.epoch, path.height).unwrap()
        });

        if let Some(pos) = pos {
            if pos > 0 {
                let prev_path = &keys[pos - 1];
                let consensus_state = self
                    .adapter
                    .get::<ClientConsensusStateSchema>(prev_path)
                    .map_err(|_| {
                        ClientError::consensus_state_not_found(client_id.clone(), height)
                    })?;
                return Ok(Some(consensus_state));
            }
        }
        Ok(None)
    }

    fn host_height(&self) -> ibc::Height {
        Height::new(0, self.adapter.get_current_height()).unwrap()
    }

    fn host_consensus_state(&self, height: ibc::Height) -> Result<AnyConsensusState, ClientError> {
        let consensus_state = self
            .consensus_states
            .get(&height.revision_height())
            .ok_or_else(|| ClientError::missing_local_consensus_state(height))?;
        Ok(AnyConsensusState::Tendermint(consensus_state.clone()))
    }

    fn pending_host_consensus_state(&self) -> Result<AnyConsensusState, ClientError> {
        let pending_height = ClientReader::host_height(self).increment();
        ClientReader::host_consensus_state(self, pending_height)
    }

    fn client_counter(&self) -> Result<u64, ClientError> {
        Ok(self.client_counter)
    }
}

impl<Adapter: IbcAdapter, Router> ClientKeeper for IbcImpl<Adapter, Router> {
    fn store_client_type(
        &mut self,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Result<(), ClientError> {
        let path = ClientTypePath(client_id);
        let res = self.adapter.set::<ClientTypeSchema>(&path, client_type);
        if res.is_ok() {
            Ok(())
        } else {
            Err(ClientError::implementation_specific())
        }
    }

    fn store_client_state(
        &mut self,
        client_id: ClientId,
        client_state: AnyClientState,
    ) -> Result<(), ClientError> {
        let path = ClientStatePath(client_id);
        let res = self.adapter.set::<ClientStateSchema>(&path, client_state);
        if res.is_ok() {
            Ok(())
        } else {
            Err(ClientError::implementation_specific())
        }
    }

    fn store_consensus_state(
        &mut self,
        client_id: ClientId,
        height: ibc::Height,
        consensus_state: AnyConsensusState,
    ) -> Result<(), ClientError> {
        let path = ClientConsensusStatePath {
            client_id,
            epoch: height.revision_number(),
            height: height.revision_height(),
        };
        let res = self
            .adapter
            .set::<ClientConsensusStateSchema>(&path, consensus_state);
        if res.is_ok() {
            Ok(())
        } else {
            Err(ClientError::implementation_specific())
        }
    }

    fn increase_client_counter(&mut self) {
        self.client_counter += 1;
    }

    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: ibc::Height,
        timestamp: Timestamp,
    ) -> Result<(), ClientError> {
        let _ = self
            .client_processed_times
            .insert((client_id, height), timestamp);
        Ok(())
    }

    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: ibc::Height,
        host_height: ibc::Height,
    ) -> Result<(), ClientError> {
        let _ = self
            .client_processed_heights
            .insert((client_id, height), host_height);
        Ok(())
    }
}

impl<Adapter: IbcAdapter, Router> ConnectionKeeper for IbcImpl<Adapter, Router> {
    fn store_connection(
        &mut self,
        connection_id: ConnectionId,
        connection_end: &ConnectionEnd,
    ) -> Result<(), ConnectionError> {
        self.adapter
            .set::<ConnectionEndSchema>(&ConnectionsPath(connection_id), connection_end.clone())
            .map_err(|_| ConnectionError::implementation_specific())?;
        Ok(())
    }

    fn store_connection_to_client(
        &mut self,
        connection_id: ConnectionId,
        client_id: &ClientId,
    ) -> Result<(), ConnectionError> {
        let path = ClientConnectionsPath(client_id.clone());
        let mut conn_ids: Vec<ConnectionId> = self
            .adapter
            .get::<ConnectionIdsSchema>(&path)
            .unwrap_or_default();
        conn_ids.push(connection_id);
        self.adapter
            .set::<ConnectionIdsSchema>(&path, conn_ids)
            .map_err(|_| ConnectionError::implementation_specific())
            .map(|_| ())
    }

    fn increase_connection_counter(&mut self) {
        self.conn_counter += 1;
    }
}

impl<Adapter: IbcAdapter, Router> ConnectionReader for IbcImpl<Adapter, Router> {
    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ConnectionError> {
        self.adapter
            .get::<ConnectionEndSchema>(&ConnectionsPath(conn_id.clone()))
            .map_err(|_| ConnectionError::implementation_specific())
    }

    fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, ConnectionError> {
        ClientReader::client_state(self, client_id).map_err(ConnectionError::ics02_client)
    }

    fn host_current_height(&self) -> ibc::Height {
        ClientReader::host_height(self)
    }

    fn host_oldest_height(&self) -> ibc::Height {
        Height::new(0, 1).unwrap()
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        CommitmentPrefix::try_from("ibc".as_bytes().to_vec()).unwrap()
    }

    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: ibc::Height,
    ) -> Result<AnyConsensusState, ConnectionError> {
        ClientReader::consensus_state(self, client_id, height)
            .map_err(ConnectionError::ics02_client)
    }

    fn host_consensus_state(
        &self,
        height: ibc::Height,
    ) -> Result<AnyConsensusState, ConnectionError> {
        ClientReader::host_consensus_state(self, height).map_err(ConnectionError::ics02_client)
    }

    fn connection_counter(&self) -> Result<u64, ConnectionError> {
        Ok(self.conn_counter)
    }
}

impl<Adapter: IbcAdapter, Router> PortReader for IbcImpl<Adapter, Router> {
    fn lookup_module_by_port(&self, port_id: &PortId) -> Result<ModuleId, PortError> {
        self.port_to_module_map
            .get(port_id)
            .ok_or_else(|| PortError::unknown_port(port_id.clone()))
            .map(Clone::clone)
    }
}

impl<Adapter: IbcAdapter, Router> ChannelKeeper for IbcImpl<Adapter, Router> {
    fn store_packet_commitment(
        &mut self,
        key: (PortId, ChannelId, Sequence),
        commitment: PacketCommitment,
    ) -> Result<(), ChannelError> {
        let path = CommitmentsPath {
            port_id:    key.0,
            channel_id: key.1,
            sequence:   key.2,
        };
        let res = self
            .adapter
            .set::<PacketCommitmentSchema>(&path, commitment);
        if res.is_ok() {
            Ok(())
        } else {
            Err(ChannelError::implementation_specific())
        }
    }

    fn delete_packet_commitment(
        &mut self,
        key: (PortId, ChannelId, Sequence),
    ) -> Result<(), ChannelError> {
        let path = CommitmentsPath {
            port_id:    key.0,
            channel_id: key.1,
            sequence:   key.2,
        };
        let res = self
            .adapter
            .set::<PacketCommitmentSchema>(&path, vec![].into());
        if res.is_ok() {
            Ok(())
        } else {
            Err(ChannelError::implementation_specific())
        }
    }

    fn store_packet_receipt(
        &mut self,
        key: (PortId, ChannelId, Sequence),
        _receipt: Receipt,
    ) -> Result<(), ChannelError> {
        let path = ReceiptsPath {
            port_id:    key.0,
            channel_id: key.1,
            sequence:   key.2,
        };
        let res = self.adapter.set::<ReceiptSchema>(&path, ());
        if res.is_ok() {
            Ok(())
        } else {
            Err(ChannelError::implementation_specific())
        }
    }

    fn store_packet_acknowledgement(
        &mut self,
        key: (PortId, ChannelId, Sequence),
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), ChannelError> {
        let path = AcksPath {
            port_id:    key.0,
            channel_id: key.1,
            sequence:   key.2,
        };
        let res = self
            .adapter
            .set::<AcknowledgementCommitmentSchema>(&path, ack_commitment);
        if res.is_ok() {
            Ok(())
        } else {
            Err(ChannelError::implementation_specific())
        }
    }

    fn delete_packet_acknowledgement(
        &mut self,
        key: (PortId, ChannelId, Sequence),
    ) -> Result<(), ChannelError> {
        self.store_packet_acknowledgement(key, vec![].into())
    }

    fn store_connection_channels(
        &mut self,
        _conn_id: ConnectionId,
        _port_channel_id: &(PortId, ChannelId),
    ) -> Result<(), ChannelError> {
        todo!()
    }

    fn store_channel(
        &mut self,
        (port_id, chan_id): (PortId, ChannelId),
        channel_end: &ibc::core::ics04_channel::channel::ChannelEnd,
    ) -> Result<(), ChannelError> {
        let path = ChannelEndsPath(port_id, chan_id);
        let res = self
            .adapter
            .set::<ChannelEndSchema>(&path, channel_end.clone());
        if res.is_ok() {
            Ok(())
        } else {
            Err(ChannelError::implementation_specific())
        }
    }

    fn store_next_sequence_send(
        &mut self,
        (port_id, chan_id): (PortId, ChannelId),
        seq: Sequence,
    ) -> Result<(), ChannelError> {
        let path = SeqSendsPath(port_id, chan_id);
        let res = self.adapter.set::<SeqSendsSchema>(&path, seq);
        if res.is_ok() {
            Ok(())
        } else {
            Err(ChannelError::implementation_specific())
        }
    }

    fn store_next_sequence_recv(
        &mut self,
        (port_id, chan_id): (PortId, ChannelId),
        seq: Sequence,
    ) -> Result<(), ChannelError> {
        let path = SeqRecvsPath(port_id, chan_id);
        let res = self.adapter.set::<SeqRecvsSchema>(&path, seq);
        if res.is_ok() {
            Ok(())
        } else {
            Err(ChannelError::implementation_specific())
        }
    }

    fn store_next_sequence_ack(
        &mut self,
        (port_id, chan_id): (PortId, ChannelId),
        seq: Sequence,
    ) -> Result<(), ChannelError> {
        let path = SeqAcksPath(port_id, chan_id);
        let res = self.adapter.set::<SeqAcksSchema>(&path, seq);
        if res.is_ok() {
            Ok(())
        } else {
            Err(ChannelError::implementation_specific())
        }
    }

    fn increase_channel_counter(&mut self) {
        self.channel_counter += 1;
    }
}

impl<Adapter: IbcAdapter, Router> ChannelReader for IbcImpl<Adapter, Router> {
    fn channel_end(
        &self,
        (port_id, chan_id): &(PortId, ChannelId),
    ) -> Result<ChannelEnd, ChannelError> {
        let path = ChannelEndsPath(port_id.clone(), chan_id.clone());
        match self.adapter.get::<ChannelEndSchema>(&path) {
            Ok(end) => Ok(end),
            Err(_) => Err(ChannelError::implementation_specific()),
        }
    }

    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ChannelError> {
        let path = ConnectionsPath(conn_id.clone());
        match self.adapter.get::<ConnectionEndSchema>(&path) {
            Ok(end) => Ok(end),
            Err(_) => Err(ChannelError::implementation_specific()),
        }
    }

    fn connection_channels(
        &self,
        _cid: &ConnectionId,
    ) -> Result<Vec<(PortId, ChannelId)>, ChannelError> {
        unimplemented!()
    }

    fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, ChannelError> {
        let path = ClientStatePath(client_id.clone());
        match self.adapter.get::<ClientStateSchema>(&path) {
            Ok(state) => Ok(state),
            Err(_) => Err(ChannelError::implementation_specific()),
        }
    }

    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: ibc::Height,
    ) -> Result<AnyConsensusState, ChannelError> {
        let path = ClientConsensusStatePath {
            client_id: client_id.clone(),
            epoch:     height.revision_number(),
            height:    height.revision_height(),
        };
        match self.adapter.get::<ClientConsensusStateSchema>(&path) {
            Ok(state) => Ok(state),
            Err(_) => Err(ChannelError::implementation_specific()),
        }
    }

    fn get_next_sequence_send(
        &self,
        (port_id, chan_id): &(PortId, ChannelId),
    ) -> Result<Sequence, ChannelError> {
        let path = SeqSendsPath(port_id.clone(), chan_id.clone());
        match self.adapter.get::<SeqSendsSchema>(&path) {
            Ok(seq) => Ok(seq),
            Err(_) => Err(ChannelError::implementation_specific()),
        }
    }

    fn get_next_sequence_recv(
        &self,
        _port_channel_id: &(PortId, ChannelId),
    ) -> Result<Sequence, ChannelError> {
        unimplemented!()
    }

    fn get_next_sequence_ack(
        &self,
        _port_channel_id: &(PortId, ChannelId),
    ) -> Result<Sequence, ChannelError> {
        unimplemented!()
    }

    fn get_packet_commitment(
        &self,
        _key: &(PortId, ChannelId, Sequence),
    ) -> Result<PacketCommitment, ChannelError> {
        unimplemented!()
    }

    fn get_packet_receipt(
        &self,
        _key: &(PortId, ChannelId, Sequence),
    ) -> Result<Receipt, ChannelError> {
        unimplemented!()
    }

    fn get_packet_acknowledgement(
        &self,
        _key: &(PortId, ChannelId, Sequence),
    ) -> Result<AcknowledgementCommitment, ChannelError> {
        unimplemented!()
    }

    fn hash(&self, value: Vec<u8>) -> Vec<u8> {
        Hasher::digest(value).as_bytes().to_vec()
    }

    fn host_height(&self) -> ibc::Height {
        Height::new(0, self.adapter.get_current_height()).unwrap()
    }

    fn host_consensus_state(
        &self,
        _height: ibc::Height,
    ) -> Result<AnyConsensusState, ChannelError> {
        unimplemented!()
    }

    fn pending_host_consensus_state(&self) -> Result<AnyConsensusState, ChannelError> {
        unimplemented!()
    }

    fn client_update_time(
        &self,
        _client_id: &ClientId,
        _height: ibc::Height,
    ) -> Result<Timestamp, ChannelError> {
        unimplemented!()
    }

    fn client_update_height(
        &self,
        _client_id: &ClientId,
        _height: ibc::Height,
    ) -> Result<ibc::Height, ChannelError> {
        unimplemented!()
    }

    fn channel_counter(&self) -> Result<u64, ChannelError> {
        unimplemented!()
    }

    fn max_expected_time_per_block(&self) -> std::time::Duration {
        unimplemented!()
    }
}

impl<Adapter: IbcAdapter> Ics26Context for IbcImpl<Adapter, IbcRouter> {
    type Router = IbcRouter;

    fn router(&self) -> &Self::Router {
        &self.router
    }

    fn router_mut(&mut self) -> &mut Self::Router {
        &mut self.router
    }
}

pub struct IbcRouter {}

impl Router for IbcRouter {
    fn get_route_mut(&mut self, _module_id: &impl Borrow<ModuleId>) -> Option<&mut dyn Module> {
        todo!()
    }

    fn has_route(&self, _module_id: &impl Borrow<ModuleId>) -> bool {
        todo!()
    }
}

pub struct Ibc {

}

impl CrossChainCodec for Ibc {
    fn get<S: StoreSchema>(&self, key: &S::Key) -> ProtocolResult<S::Value> {
        todo!()
    }
    fn set<S: StoreSchema>(&self, key: &S::Key, value: S::Value) -> ProtocolResult<()> {
        todo!()
    }
    fn get_all_keys<S: StoreSchema>(&self) -> &[S::Key] {
        todo!()
    }   
}

impl IbcAdapter for Ibc {
    fn get_current_height(&self) -> u64 {
        todo!()
    }
}