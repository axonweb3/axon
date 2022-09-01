pub use ckb_crosschain_schema::{CkbCrossChainSchema, MonitorCkbNumberSchema};

use protocol::traits::{StorageCategory, StorageSchema};
use protocol::types::{
    Block, Bytes, DBBytes, Hash, HashWithDirection, Header, Proof, Receipt, SignedTransaction,
};

use crate::hash_key::{BlockKey, CommonHashKey};

macro_rules! impl_storage_schema_for {
    ($name: ident, $key: ty, $val: ty, $category: ident) => {
        pub struct $name;

        impl StorageSchema for $name {
            type Key = $key;
            type Value = $val;

            fn category() -> StorageCategory {
                StorageCategory::$category
            }
        }
    };
}

impl_storage_schema_for!(
    TransactionSchema,
    CommonHashKey,
    SignedTransaction,
    SignedTransaction
);
impl_storage_schema_for!(
    TransactionBytesSchema,
    CommonHashKey,
    DBBytes,
    SignedTransaction
);
impl_storage_schema_for!(BlockSchema, BlockKey, Block, Block);
impl_storage_schema_for!(BlockHeaderSchema, BlockKey, Header, BlockHeader);
impl_storage_schema_for!(BlockHashNumberSchema, Hash, u64, HashHeight);
impl_storage_schema_for!(ReceiptSchema, CommonHashKey, Receipt, Receipt);
impl_storage_schema_for!(ReceiptBytesSchema, CommonHashKey, DBBytes, Receipt);
impl_storage_schema_for!(TxHashNumberSchema, Hash, u64, HashHeight);
impl_storage_schema_for!(LatestBlockSchema, Hash, Block, Block);
impl_storage_schema_for!(LatestProofSchema, Hash, Proof, Block);
impl_storage_schema_for!(OverlordWalSchema, Hash, Bytes, Wal);
impl_storage_schema_for!(EvmCodeSchema, Hash, Bytes, Code);
impl_storage_schema_for!(EvmCodeAddressSchema, Hash, Hash, Code);

mod ckb_crosschain_schema {
    use super::*;

    impl_storage_schema_for!(CkbCrossChainSchema, Hash, HashWithDirection, CkbCrossChain);
    impl_storage_schema_for!(MonitorCkbNumberSchema, Hash, u64, CkbCrossChain);
}

#[cfg(feature = "ibc")]
pub mod ibc_crosschain_schema {
    use super::*;

    use cosmos_ibc::core::{
        ics02_client::{
            client_consensus::AnyConsensusState, client_state::AnyClientState,
            client_type::ClientType,
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
        },
    };

    use protocol::codec::crosschain::ibc::IbcWrapper;

    impl_storage_schema_for!(
        ClientTypeSchema,
        IbcWrapper<ClientTypePath>,
        IbcWrapper<ClientType>,
        IbcCrossChain
    );
    impl_storage_schema_for!(
        ClientStateSchema,
        IbcWrapper<ClientStatePath>,
        IbcWrapper<AnyClientState>,
        IbcCrossChain
    );
    impl_storage_schema_for!(
        ClientConsensusStateSchema,
        IbcWrapper<ClientConsensusStatePath>,
        IbcWrapper<AnyConsensusState>,
        IbcCrossChain
    );
    impl_storage_schema_for!(
        ConnectionEndSchema,
        IbcWrapper<ConnectionsPath>,
        IbcWrapper<ConnectionEnd>,
        IbcCrossChain
    );
    impl_storage_schema_for!(
        ConnectionIdsSchema,
        IbcWrapper<ClientConnectionsPath>,
        IbcWrapper<Vec<ConnectionId>>,
        IbcCrossChain
    );
    impl_storage_schema_for!(
        ChannelEndSchema,
        IbcWrapper<ChannelEndsPath>,
        IbcWrapper<ChannelEnd>,
        IbcCrossChain
    );
    impl_storage_schema_for!(
        SeqSendsSchema,
        IbcWrapper<SeqSendsPath>,
        IbcWrapper<Sequence>,
        IbcCrossChain
    );
    impl_storage_schema_for!(
        SeqRecvsSchema,
        IbcWrapper<SeqRecvsPath>,
        IbcWrapper<Sequence>,
        IbcCrossChain
    );
    impl_storage_schema_for!(
        SeqAcksSchema,
        IbcWrapper<SeqAcksPath>,
        IbcWrapper<Sequence>,
        IbcCrossChain
    );
    impl_storage_schema_for!(
        PacketCommitmentSchema,
        IbcWrapper<CommitmentsPath>,
        IbcWrapper<PacketCommitment>,
        IbcCrossChain
    );
    impl_storage_schema_for!(
        AcknowledgementCommitmentSchema,
        IbcWrapper<AcksPath>,
        IbcWrapper<AcknowledgementCommitment>,
        IbcCrossChain
    );
    impl_storage_schema_for!(
        ReceiptSchema,
        IbcWrapper<ReceiptsPath>,
        IbcWrapper<()>,
        IbcCrossChain
    );
}
