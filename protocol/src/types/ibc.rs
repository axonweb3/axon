use cosmos_ibc::core::ics02_client::client_consensus::{AnyConsensusState, ConsensusState};
use cosmos_ibc::core::ics02_client::client_state::{AnyClientState, ClientState};
use cosmos_ibc::core::ics02_client::header::{AnyHeader, Header as IbcHeader};
use cosmos_ibc::core::ics02_client::{client_type::ClientType, height::Height};
use cosmos_ibc::core::ics23_commitment::commitment::CommitmentRoot;
use cosmos_ibc::core::ics24_host::identifier::ChainId;
use cosmos_ibc::timestamp::Timestamp;

use crate::types::{Hash, Header, MerkleRoot};
use crate::ProtocolError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsensusStateWithHeight {
    pub height:               u64,
    pub round:                u64,
    pub timestamp:            u64,
    pub state_root:           CommitmentRoot,
    pub next_validators_hash: Hash,
}

impl ConsensusState for ConsensusStateWithHeight {
    type Error = ProtocolError;

    fn client_type(&self) -> ClientType {
        todo!()
    }

    fn root(&self) -> &CommitmentRoot {
        &self.state_root
    }

    fn wrap_any(self) -> AnyConsensusState {
        todo!()
    }
}

impl ConsensusStateWithHeight {
    pub fn new(
        height: u64,
        round: u64,
        timestamp: u64,
        state_root: MerkleRoot,
        next_validators_hash: Hash,
    ) -> Self {
        ConsensusStateWithHeight {
            height,
            round,
            timestamp,
            state_root: CommitmentRoot::from_bytes(state_root.as_bytes()),
            next_validators_hash,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxonClientState {
    pub chian_id:      u64,
    pub latest_height: u64,
    pub frozen_height: u64,
    pub is_frozen:     bool,
}

impl ClientState for AxonClientState {
    type UpgradeOptions = ();

    fn chain_id(&self) -> ChainId {
        todo!()
    }

    fn client_type(&self) -> ClientType {
        todo!()
    }

    fn latest_height(&self) -> Height {
        todo!()
    }

    fn frozen_height(&self) -> Option<Height> {
        todo!()
    }

    fn upgrade(
        self,
        _upgrade_height: Height,
        _upgrade_options: Self::UpgradeOptions,
        _chain_id: ChainId,
    ) -> Self {
        todo!()
    }

    fn wrap_any(self) -> AnyClientState {
        todo!()
    }
}

impl IbcHeader for Header {
    fn client_type(&self) -> ClientType {
        todo!()
    }

    fn height(&self) -> Height {
        todo!()
    }

    fn timestamp(&self) -> Timestamp {
        todo!()
    }

    fn wrap_any(self) -> AnyHeader {
        todo!()
    }
}
