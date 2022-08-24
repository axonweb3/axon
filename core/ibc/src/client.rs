use ibc::core::ics02_client::{
    client_consensus::AnyConsensusState, client_def::ClientDef, client_state::AnyClientState,
    context::ClientReader, error::Error, height::Height,
};
use ibc::core::ics03_connection::connection::ConnectionEnd;
use ibc::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::ics04_channel::{channel::ChannelEnd, context::ChannelReader, packet::Sequence};
use ibc::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;

use protocol::types::{AxonClientState, ConsensusStateWithHeight, Header};

#[derive(Clone)]
pub struct IbcLightClient {}

impl ClientDef for IbcLightClient {
    type ClientState = AxonClientState;
    type ConsensusState = ConsensusStateWithHeight;
    type Header = Header;

    fn check_header_and_update_state(
        &self,
        _ctx: &dyn ClientReader,
        _client_id: ClientId,
        _client_state: Self::ClientState,
        _header: Self::Header,
    ) -> Result<(Self::ClientState, Self::ConsensusState), Error> {
        todo!()
    }

    fn verify_upgrade_and_update_state(
        &self,
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _proof_upgrade_client: RawMerkleProof,
        _proof_upgrade_consensus_state: RawMerkleProof,
    ) -> Result<(Self::ClientState, Self::ConsensusState), Error> {
        todo!()
    }

    fn verify_client_consensus_state(
        &self,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _client_id: &ClientId,
        _consensus_height: Height,
        _expected_consensus_state: &AnyConsensusState,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_connection_state(
        &self,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _connection_id: &ConnectionId,
        _expected_connection_end: &ConnectionEnd,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_channel_state(
        &self,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _expected_channel_end: &ChannelEnd,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_client_full_state(
        &self,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _client_id: &ClientId,
        _expected_client_state: &AnyClientState,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_packet_data(
        &self,
        _ctx: &dyn ChannelReader,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
        _commitment: PacketCommitment,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_packet_acknowledgement(
        &self,
        _ctx: &dyn ChannelReader,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
        _ack: AcknowledgementCommitment,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_next_sequence_recv(
        &self,
        _ctx: &dyn ChannelReader,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_packet_receipt_absence(
        &self,
        _ctx: &dyn ChannelReader,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
    ) -> Result<(), Error> {
        todo!()
    }
}
