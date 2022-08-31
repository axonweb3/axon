use ibc::core::{
    ics02_client::{
        client_consensus::AnyConsensusState, client_state::AnyClientState, client_type::ClientType,
    },
    ics03_connection::connection::ConnectionEnd,
    ics04_channel::{
        channel::ChannelEnd,
        commitment::{AcknowledgementCommitment, PacketCommitment},
        packet::Sequence,
    },
    ics24_host::{
        identifier::ConnectionId,
        path::{
            AcksPath, ChannelEndsPath, ClientConnectionsPath, ClientConsensusStatePath,
            ClientStatePath, ClientTypePath, CommitmentsPath, ConnectionsPath, ReceiptsPath,
            SeqAcksPath, SeqRecvsPath, SeqSendsPath,
        },
        Path,
    },
};
use protocol::ProtocolResult;

use crate::codec::Codec;
use crate::store::Path as StorePath;

pub trait StoreSchema {
    type Key: Into<Path>;
    type Value: Codec;
}

macro_rules! impl_store_schema {
    ($name:ident, $key:ty, $value:ty) => {
        pub struct $name;
        impl StoreSchema for $name {
            type Key = $key;
            type Value = $value;
        }
    };
}

impl_store_schema!(ClientTypeSchema, ClientTypePath, ClientType);
impl_store_schema!(ClientStateSchema, ClientStatePath, AnyClientState);
impl_store_schema!(
    ClientConsensusStateSchema,
    ClientConsensusStatePath,
    AnyConsensusState
);
impl_store_schema!(ConnectionEndSchema, ConnectionsPath, ConnectionEnd);
impl_store_schema!(
    ConnectionIdsSchema,
    ClientConnectionsPath,
    Vec<ConnectionId>
);
impl_store_schema!(ChannelEndSchema, ChannelEndsPath, ChannelEnd);
impl_store_schema!(SeqSendsSchema, SeqSendsPath, Sequence);
impl_store_schema!(SeqRecvsSchema, SeqRecvsPath, Sequence);
impl_store_schema!(SeqAcksSchema, SeqAcksPath, Sequence);
impl_store_schema!(PacketCommitmentSchema, CommitmentsPath, PacketCommitment);
impl_store_schema!(
    AcknowledgementCommitmentSchema,
    AcksPath,
    AcknowledgementCommitment
);
impl_store_schema!(ReceiptSchema, ReceiptsPath, ());

pub trait CrossChainCodec {
    fn get<S: StoreSchema>(&self, key: &S::Key) -> ProtocolResult<S::Value>;
    fn set<S: StoreSchema>(&self, key: &S::Key, value: S::Value) -> ProtocolResult<()>;
    fn get_all_keys<S: StoreSchema>(&self) -> &[S::Key];
}

pub trait IbcAdapter: CrossChainCodec {
    fn get_current_height(&self) -> u64;

    // #[inline]
    // fn get(&self, height: Height, path: &K) -> Option<V> {
    //     todo!()
    // }

    #[inline]
    fn get_keys(&self, key_prefix: &StorePath) -> Vec<StorePath> {
        todo!()
    }
}
