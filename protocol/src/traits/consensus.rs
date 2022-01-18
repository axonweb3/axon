use std::collections::HashMap;

use async_trait::async_trait;
use bytes::Bytes;
use creep::Context;

use crate::types::{
    Address, Block, BlockNumber, ExecResp, Hash, Header, Hex, Log, MerkleRoot, Proof, Proposal,
    Receipt, SignedTransaction, Validator, U256,
};
use crate::ProtocolResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageTarget {
    Broadcast,
    Specified(Bytes),
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub chain_id:     u64,
    pub self_pub_key: Bytes,
    pub self_address: Address,
}

#[async_trait]
pub trait Consensus: Send + Sync {
    /// Network set a received signed proposal to consensus.
    async fn set_proposal(&self, ctx: Context, proposal: Vec<u8>) -> ProtocolResult<()>;

    /// Network set a received signed vote to consensus.
    async fn set_vote(&self, ctx: Context, vote: Vec<u8>) -> ProtocolResult<()>;

    /// Network set a received quorum certificate to consensus.
    async fn set_qc(&self, ctx: Context, qc: Vec<u8>) -> ProtocolResult<()>;

    /// Network set a received signed choke to consensus.
    async fn set_choke(&self, ctx: Context, choke: Vec<u8>) -> ProtocolResult<()>;
}

#[async_trait]
pub trait Synchronization: Send + Sync {
    async fn receive_remote_block(&self, ctx: Context, remote_height: u64) -> ProtocolResult<()>;
}

#[async_trait]
pub trait SynchronizationAdapter: CommonConsensusAdapter + Send + Sync {
    fn update_status(
        &self,
        ctx: Context,
        height: u64,
        consensus_interval: u64,
        propose_ratio: u64,
        prevote_ratio: u64,
        precommit_ratio: u64,
        brake_ratio: u64,
        validators: Vec<Validator>,
    ) -> ProtocolResult<()>;

    /// Pull some blocks from other nodes from `begin` to `end`.
    async fn get_block_from_remote(
        &self,
        ctx: Context,
        number: BlockNumber,
    ) -> ProtocolResult<Block>;

    /// Pull signed transactions corresponding to the given hashes from other
    /// nodes.
    async fn get_txs_from_remote(
        &self,
        ctx: Context,
        number: BlockNumber,
        hashes: &[Hash],
    ) -> ProtocolResult<Vec<SignedTransaction>>;

    async fn get_proof_from_remote(
        &self,
        ctx: Context,
        number: BlockNumber,
    ) -> ProtocolResult<Proof>;
}

#[async_trait]
pub trait CommonConsensusAdapter: Send + Sync {
    /// Save a block to the database.
    async fn save_block(&self, ctx: Context, block: Block) -> ProtocolResult<()>;

    async fn save_proof(&self, ctx: Context, proof: Proof) -> ProtocolResult<()>;

    /// Save some signed transactions to the database.
    async fn save_signed_txs(
        &self,
        ctx: Context,
        block_height: u64,
        signed_txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<()>;

    async fn save_receipts(
        &self,
        ctx: Context,
        height: u64,
        receipts: Vec<Receipt>,
    ) -> ProtocolResult<()>;

    /// Flush the given transactions in the mempool.
    async fn flush_mempool(&self, ctx: Context, ordered_tx_hashes: &[Hash]) -> ProtocolResult<()>;

    /// Get a block corresponding to the given height.
    async fn get_block_by_number(&self, ctx: Context, height: u64) -> ProtocolResult<Block>;

    async fn get_block_header_by_number(&self, ctx: Context, height: u64)
        -> ProtocolResult<Header>;

    /// Get the current height from storage.
    async fn get_current_number(&self, ctx: Context) -> ProtocolResult<u64>;

    async fn get_txs_from_storage(
        &self,
        ctx: Context,
        tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<SignedTransaction>>;

    /// Execute some transactions.
    async fn exec(
        &self,
        ctx: Context,
        last_state_root: Hash,
        proposal: &Proposal,
        signed_txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<ExecResp>;

    async fn broadcast_number(&self, ctx: Context, height: u64) -> ProtocolResult<()>;

    fn set_args(
        &self,
        context: Context,
        state_root: MerkleRoot,
        timeout_gap: u64,
        gas_limit: u64,
        max_tx_size: u64,
    );

    fn tag_consensus(&self, ctx: Context, peer_ids: Vec<Bytes>) -> ProtocolResult<()>;

    async fn verify_proof(&self, ctx: Context, block: Block, proof: Proof) -> ProtocolResult<()>;

    async fn verify_block_header(&self, ctx: Context, block: &Proposal) -> ProtocolResult<()>;

    async fn notify_block_logs(
        &self,
        ctx: Context,
        block_number: u64,
        block_hash: Hash,
        logs: &[Vec<Log>],
    ) -> ProtocolResult<()>;

    async fn notify_checkpoint(
        &self,
        ctx: Context,
        block: Block,
        proof: Proof,
    ) -> ProtocolResult<()>;

    fn verify_proof_signature(
        &self,
        ctx: Context,
        block_height: u64,
        vote_hash: Bytes,
        aggregated_signature_bytes: Bytes,
        vote_pubkeys: Vec<Hex>,
    ) -> ProtocolResult<()>;

    fn verify_proof_weight(
        &self,
        ctx: Context,
        block_height: u64,
        weight_map: HashMap<Bytes, u32>,
        signed_voters: Vec<Bytes>,
    ) -> ProtocolResult<()>;
}

#[async_trait]
pub trait ConsensusAdapter: CommonConsensusAdapter + Send + Sync {
    /// Get some transaction hashes of the given height. The amount of the
    /// transactions is limited by the given cycle limit and return a
    /// `MixedTxHashes` struct.
    async fn get_txs_from_mempool(
        &self,
        ctx: Context,
        height: u64,
        gas_limit: U256,
        tx_num_limit: u64,
    ) -> ProtocolResult<Vec<Hash>>;

    /// Synchronous signed transactions.
    async fn sync_txs(&self, ctx: Context, propose_txs: Vec<Hash>) -> ProtocolResult<()>;

    /// Get the signed transactions corresponding to the given hashes.
    async fn get_full_txs(
        &self,
        ctx: Context,
        order_txs: &[Hash],
    ) -> ProtocolResult<Vec<SignedTransaction>>;

    /// Consensus transmit a message to the given target.
    async fn transmit(
        &self,
        ctx: Context,
        msg: Vec<u8>,
        end: &str,
        target: MessageTarget,
    ) -> ProtocolResult<()>;

    /// Get the current height from storage.
    async fn get_current_number(&self, ctx: Context) -> ProtocolResult<u64>;

    /// Pull some blocks from other nodes from `begin` to `end`.
    async fn pull_block(&self, ctx: Context, number: u64, end: &str) -> ProtocolResult<Block>;

    async fn verify_txs(&self, ctx: Context, number: u64, txs: &[Hash]) -> ProtocolResult<()>;
}
