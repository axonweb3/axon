use std::{
    fmt::{Display, Formatter},
    ops::Deref,
    str::{from_utf8, FromStr},
};

use cosmos_ibc::core::ics02_client::client_consensus::{AnyConsensusState, ConsensusState};
use cosmos_ibc::core::ics02_client::client_state::{AnyClientState, ClientState};
use cosmos_ibc::core::ics02_client::header::{AnyHeader, Header as IbcHeader};
use cosmos_ibc::core::ics02_client::{client_type::ClientType, height::Height};
use cosmos_ibc::core::ics23_commitment::commitment::CommitmentRoot;
use cosmos_ibc::core::ics24_host::{
    identifier::ChainId, path, validate::validate_identifier, Path as IbcPath,
};
use cosmos_ibc::timestamp::Timestamp;

use crate::types::{Hash, Header, MerkleRoot};
use crate::{ProtocolError, ProtocolErrorKind};

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

/// A newtype representing a valid ICS024 identifier.
/// Implements `Deref<Target=String>`.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Identifier(String);

impl Identifier {
    /// Identifiers MUST be non-empty (of positive integer length).
    /// Identifiers MUST consist of characters in one of the following
    /// categories only:
    /// * Alphanumeric
    /// * `.`, `_`, `+`, `-`, `#`
    /// * `[`, `]`, `<`, `>`
    fn validate(s: impl AsRef<str>) -> Result<(), ProtocolError> {
        let s = s.as_ref();

        // give a `min` parameter of 0 here to allow id's of arbitrary
        // length as inputs; `validate_identifier` itself checks for
        // empty inputs and returns an error as appropriate
        validate_identifier(s, 0, s.len())
            .map_err(|e| ProtocolError::new(ProtocolErrorKind::Ibc, Box::new(e)))
    }
}

impl Deref for Identifier {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for Identifier {
    type Error = ProtocolError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Identifier::validate(&s).map(|_| Self(s))
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A newtype representing a valid ICS024 `Path`.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Path(Vec<Identifier>);

impl TryFrom<String> for Path {
    type Error = ProtocolError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let mut identifiers = vec![];
        let parts = s.split('/'); // split will never return an empty iterator
        for part in parts {
            identifiers.push(Identifier::try_from(part.to_owned())?);
        }
        Ok(Self(identifiers))
    }
}

impl TryFrom<&[u8]> for Path {
    type Error = ProtocolError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let s = from_utf8(value)
            .map_err(|e| ProtocolError::new(ProtocolErrorKind::Ibc, Box::new(e)))?;
        s.to_owned().try_into()
    }
}

impl From<Identifier> for Path {
    fn from(id: Identifier) -> Self {
        Self(vec![id])
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|iden| iden.as_str().to_owned())
                .collect::<Vec<String>>()
                .join("/")
        )
    }
}

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

/// Store height to query
#[derive(Debug, Copy, Clone)]
pub enum StoreHeight {
    Pending,
    // Latest,
    // Stable(RawHeight),
}
