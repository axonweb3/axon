use crate::types::{
    Block, Bytes, Direction, Hash, HashWithDirection, Header, Proof, Receipt, RequestTxHashes,
    SignedTransaction, H256,
};
use crate::{async_trait, codec::ProtocolCodec, traits::Context, Display, ProtocolResult};

#[derive(Debug, Copy, Clone, Display)]
pub enum StorageCategory {
    Block,
    BlockHeader,
    Receipt,
    SignedTransaction,
    Wal,
    HashHeight,
    Code,
    CkbCrossChain,
    IbcCrossChain,
}

pub type StorageIterator<'a, S> = Box<
    dyn Iterator<Item = ProtocolResult<(<S as StorageSchema>::Key, <S as StorageSchema>::Value)>>
        + 'a,
>;

pub trait StorageSchema {
    type Key: ProtocolCodec + Send;
    type Value: ProtocolCodec + Send;

    fn category() -> StorageCategory;
}

pub trait IntoIteratorByRef<S: StorageSchema> {
    fn ref_to_iter<'a, 'b: 'a>(&'b self) -> StorageIterator<'a, S>;
}

#[async_trait]
pub trait CommonStorage: Send + Sync {
    async fn insert_block(&self, ctx: Context, block: Block) -> ProtocolResult<()>;

    async fn get_block(&self, ctx: Context, height: u64) -> ProtocolResult<Option<Block>>;

    async fn get_block_header(&self, ctx: Context, height: u64) -> ProtocolResult<Option<Header>>;

    async fn set_block(&self, _ctx: Context, block: Block) -> ProtocolResult<()>;

    async fn remove_block(&self, ctx: Context, height: u64) -> ProtocolResult<()>;

    async fn get_latest_block(&self, ctx: Context) -> ProtocolResult<Block>;

    async fn set_latest_block(&self, ctx: Context, block: Block) -> ProtocolResult<()>;

    async fn get_latest_block_header(&self, ctx: Context) -> ProtocolResult<Header>;
}

#[async_trait]
pub trait Storage: CommonStorage + CkbCrossChainStorage {
    async fn insert_transactions(
        &self,
        ctx: Context,
        block_height: u64,
        signed_txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<()>;

    async fn get_block_by_hash(
        &self,
        ctx: Context,
        block_hash: &Hash,
    ) -> ProtocolResult<Option<Block>>;

    async fn get_transactions(
        &self,
        ctx: Context,
        block_height: u64,
        hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<SignedTransaction>>>;

    async fn get_transaction_by_hash(
        &self,
        ctx: Context,
        hash: &Hash,
    ) -> ProtocolResult<Option<SignedTransaction>>;

    async fn insert_receipts(
        &self,
        ctx: Context,
        block_height: u64,
        receipts: Vec<Receipt>,
    ) -> ProtocolResult<()>;

    async fn insert_code(
        &self,
        ctx: Context,
        code_address: H256,
        code_hash: Hash,
        code: Bytes,
    ) -> ProtocolResult<()>;

    async fn get_code_by_hash(&self, ctx: Context, hash: &Hash) -> ProtocolResult<Option<Bytes>>;

    async fn get_code_by_address(
        &self,
        _ctx: Context,
        address: &H256,
    ) -> ProtocolResult<Option<Bytes>>;

    async fn get_receipt_by_hash(
        &self,
        ctx: Context,
        hash: &Hash,
    ) -> ProtocolResult<Option<Receipt>>;

    async fn get_receipts(
        &self,
        ctx: Context,
        block_height: u64,
        hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<Receipt>>>;

    async fn update_latest_proof(&self, ctx: Context, proof: Proof) -> ProtocolResult<()>;

    async fn get_latest_proof(&self, ctx: Context) -> ProtocolResult<Proof>;
}

#[async_trait]
pub trait CkbCrossChainStorage: Send + Sync {
    async fn insert_crosschain_records(
        &self,
        ctx: Context,
        reqs: RequestTxHashes,
        block_hash: Hash,
        direction: Direction,
    ) -> ProtocolResult<()>;

    async fn get_crosschain_record(
        &self,
        ctx: Context,
        hash: &Hash,
    ) -> ProtocolResult<Option<HashWithDirection>>;

    async fn update_monitor_ckb_number(&self, ctx: Context, number: u64) -> ProtocolResult<()>;

    async fn get_monitor_ckb_number(&self, ctx: Context) -> ProtocolResult<u64>;
}

#[async_trait]
pub trait MaintenanceStorage: CommonStorage {}

pub enum StorageBatchModify<S: StorageSchema> {
    Remove,
    Insert(<S as StorageSchema>::Value),
}

pub trait StorageAdapter: Send + Sync {
    fn insert<S: StorageSchema>(
        &self,
        key: <S as StorageSchema>::Key,
        val: <S as StorageSchema>::Value,
    ) -> ProtocolResult<()>;

    fn get<S: StorageSchema>(
        &self,
        key: <S as StorageSchema>::Key,
    ) -> ProtocolResult<Option<<S as StorageSchema>::Value>>;

    fn get_batch<S: StorageSchema>(
        &self,
        keys: Vec<<S as StorageSchema>::Key>,
    ) -> ProtocolResult<Vec<Option<<S as StorageSchema>::Value>>> {
        let mut vec = Vec::with_capacity(keys.len());

        for key in keys {
            vec.push(self.get::<S>(key)?);
        }

        Ok(vec)
    }

    fn remove<S: StorageSchema>(&self, key: <S as StorageSchema>::Key) -> ProtocolResult<()>;

    fn contains<S: StorageSchema>(&self, key: <S as StorageSchema>::Key) -> ProtocolResult<bool>;

    fn batch_modify<S: StorageSchema>(
        &self,
        keys: Vec<<S as StorageSchema>::Key>,
        vals: Vec<StorageBatchModify<S>>,
    ) -> ProtocolResult<()>;

    fn prepare_iter<'a, 'b: 'a, S: StorageSchema + 'static, P: AsRef<[u8]> + 'a>(
        &'b self,
        prefix: &'a P,
    ) -> ProtocolResult<Box<dyn IntoIteratorByRef<S> + 'a>>;
}

#[cfg(feature = "ibc")]
pub mod ibc {
    use cosmos_ibc::{
        core::{
            ics02_client::client_consensus::AnyConsensusState,
            ics02_client::{client_state::AnyClientState, client_type::ClientType},
            ics03_connection::connection::ConnectionEnd,
            ics04_channel::channel::ChannelEnd,
            ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment},
            ics04_channel::packet::{Receipt, Sequence},
            ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
        },
        Height,
    };

    use crate::ProtocolResult;

    pub trait IbcCrossChainStorage {
        fn get_client_type(&self, client_id: &ClientId) -> ProtocolResult<Option<ClientType>>;

        fn get_client_state(&self, client_id: &ClientId) -> ProtocolResult<Option<AnyClientState>>;

        fn get_consensus_state(
            &self,
            client_id: &ClientId,
            epoch: u64,
            height: u64,
        ) -> ProtocolResult<Option<AnyConsensusState>>;

        fn get_next_consensus_state(
            &self,
            client_id: &ClientId,
            height: Height,
        ) -> ProtocolResult<Option<AnyConsensusState>>;

        fn get_prev_consensus_state(
            &self,
            client_id: &ClientId,
            height: Height,
        ) -> ProtocolResult<Option<AnyConsensusState>>;

        fn set_client_type(
            &mut self,
            client_id: ClientId,
            client_type: ClientType,
        ) -> ProtocolResult<()>;

        fn set_client_state(
            &mut self,
            client_id: ClientId,
            client_state: AnyClientState,
        ) -> ProtocolResult<()>;

        fn set_consensus_state(
            &mut self,
            client_id: ClientId,
            height: Height,
            consensus_state: AnyConsensusState,
        ) -> ProtocolResult<()>;

        fn set_connection_end(
            &mut self,
            connection_id: ConnectionId,
            connection_end: ConnectionEnd,
        ) -> ProtocolResult<()>;

        fn set_connection_to_client(
            &mut self,
            connection_id: ConnectionId,
            client_id: &ClientId,
        ) -> ProtocolResult<()>;

        fn get_connection_end(
            &self,
            conn_id: &ConnectionId,
        ) -> ProtocolResult<Option<ConnectionEnd>>;

        fn set_packet_commitment(
            &mut self,
            key: (PortId, ChannelId, Sequence),
            commitment: PacketCommitment,
        ) -> ProtocolResult<()>;

        fn get_packet_commitment(
            &self,
            key: &(PortId, ChannelId, Sequence),
        ) -> ProtocolResult<Option<PacketCommitment>>;

        fn delete_packet_commitment(
            &mut self,
            key: (PortId, ChannelId, Sequence),
        ) -> ProtocolResult<()>;

        fn set_packet_receipt(
            &mut self,
            key: (PortId, ChannelId, Sequence),
            receipt: Receipt,
        ) -> ProtocolResult<()>;

        fn get_packet_receipt(
            &self,
            key: &(PortId, ChannelId, Sequence),
        ) -> ProtocolResult<Option<Receipt>>;

        fn set_packet_acknowledgement(
            &mut self,
            key: (PortId, ChannelId, Sequence),
            ack_commitment: AcknowledgementCommitment,
        ) -> ProtocolResult<()>;

        fn get_packet_acknowledgement(
            &self,
            key: &(PortId, ChannelId, Sequence),
        ) -> ProtocolResult<Option<AcknowledgementCommitment>>;

        fn delete_packet_acknowledgement(
            &mut self,
            key: (PortId, ChannelId, Sequence),
        ) -> ProtocolResult<()>;

        fn set_connection_channels(
            &mut self,
            _conn_id: ConnectionId,
            _port_channel_id: &(PortId, ChannelId),
        ) -> ProtocolResult<()>;

        fn set_channel(
            &mut self,
            port_id: PortId,
            chan_id: ChannelId,
            chan_end: ChannelEnd,
        ) -> ProtocolResult<()>;

        fn set_next_sequence_send(
            &mut self,
            port_id: PortId,
            chan_id: ChannelId,
            seq: Sequence,
        ) -> ProtocolResult<()>;

        fn set_next_sequence_recv(
            &mut self,
            port_id: PortId,
            chan_id: ChannelId,
            seq: Sequence,
        ) -> ProtocolResult<()>;

        fn set_next_sequence_ack(
            &mut self,
            port_id: PortId,
            chan_id: ChannelId,
            seq: Sequence,
        ) -> ProtocolResult<()>;

        fn get_channel_end(
            &self,
            port_channel_id: &(PortId, ChannelId),
        ) -> ProtocolResult<Option<ChannelEnd>>;

        fn get_next_sequence_send(
            &self,
            port_channel_id: &(PortId, ChannelId),
        ) -> ProtocolResult<Option<Sequence>>;

        fn get_next_sequence_recv(
            &self,
            port_channel_id: &(PortId, ChannelId),
        ) -> ProtocolResult<Option<Sequence>>;

        fn get_next_sequence_ack(
            &self,
            port_channel_id: &(PortId, ChannelId),
        ) -> ProtocolResult<Option<Sequence>>;
    }
}
