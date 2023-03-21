use std::collections::HashMap;
use std::sync::Arc;

use core_executor::system_contract::metadata::MetadataHandle;
use overlord::types::{Node, OverlordMsg, Vote, VoteType};
use overlord::{extract_voters, Crypto, OverlordHandler};
use parking_lot::RwLock;

use common_apm::Instant;
use common_apm_derive::trace_span;
use core_executor::{AxonExecutor, AxonExecutorAdapter};
use core_network::{PeerId, PeerIdExt};
use protocol::traits::{
    CommonConsensusAdapter, ConsensusAdapter, Context, Executor, Gossip, MemPool, MessageTarget,
    Network, PeerTrust, Priority, Rpc, Storage, SynchronizationAdapter,
};
use protocol::types::{
    BatchSignedTxs, Block, BlockNumber, Bytes, ExecResp, Hash, Header, Hex, MerkleRoot, Metadata,
    PackedTxHashes, Proof, Proposal, Receipt, SignedTransaction, Validator, U256,
};
use protocol::{async_trait, tokio::task, trie, ProtocolResult};

use crate::consensus::gen_overlord_status;
use crate::message::{
    BROADCAST_HEIGHT, RPC_SYNC_PULL_BLOCK, RPC_SYNC_PULL_PROOF, RPC_SYNC_PULL_TXS,
};
use crate::types::PullTxsRequest;
use crate::util::{convert_hex_to_bls_pubkeys, OverlordCrypto};
use crate::BlockHeaderField::PreviousBlockHash;
use crate::BlockProofField::{BitMap, HashMismatch, HeightMismatch, Signature, WeightNotFound};
use crate::{BlockProofField, ConsensusError};

pub struct OverlordConsensusAdapter<
    M: MemPool,
    N: Rpc + PeerTrust + Gossip + 'static,
    S: Storage,
    DB: trie::DB,
> {
    network: Arc<N>,
    mempool: Arc<M>,

    storage:          Arc<S>,
    trie_db:          Arc<DB>,
    metadata:         Arc<MetadataHandle>,
    overlord_handler: RwLock<Option<OverlordHandler<Proposal>>>,
    crypto:           Arc<OverlordCrypto>,
}

#[async_trait]
impl<M, N, S, DB> ConsensusAdapter for OverlordConsensusAdapter<M, N, S, DB>
where
    M: MemPool + 'static,
    N: Network + Rpc + PeerTrust + Gossip + 'static,
    S: Storage + 'static,
    DB: trie::DB + 'static,
{
    #[trace_span(kind = "consensus.adapter")]
    async fn get_txs_from_mempool(
        &self,
        ctx: Context,
        _number: u64,
        gas_limit: U256,
        tx_num_limit: u64,
    ) -> ProtocolResult<PackedTxHashes> {
        self.mempool.package(ctx, gas_limit, tx_num_limit).await
    }

    #[trace_span(kind = "consensus.adapter", logs = "{txs_len: txs.len()}")]
    async fn get_full_txs(
        &self,
        ctx: Context,
        txs: &[Hash],
    ) -> ProtocolResult<Vec<SignedTransaction>> {
        self.mempool.get_full_txs(ctx, None, txs).await
    }

    #[trace_span(kind = "consensus.adapter")]
    async fn transmit(
        &self,
        ctx: Context,
        msg: Vec<u8>,
        end: &str,
        target: MessageTarget,
    ) -> ProtocolResult<()> {
        match target {
            MessageTarget::Broadcast => self.network.broadcast(ctx, end, msg, Priority::High).await,

            MessageTarget::Specified(pub_key) => {
                let peer_id_bytes = PeerId::from_pubkey_bytes(pub_key)?.into_bytes_ext();

                self.network
                    .multicast(ctx, end, [peer_id_bytes], msg, Priority::High)
                    .await
            }
        }
    }

    /// Get the current number from storage.
    #[trace_span(kind = "consensus.adapter")]
    async fn get_current_number(&self, ctx: Context) -> ProtocolResult<u64> {
        let header = self.storage.get_latest_block_header(ctx).await?;
        Ok(header.number)
    }

    #[trace_span(kind = "consensus.adapter")]
    async fn pull_block(&self, ctx: Context, number: u64, end: &str) -> ProtocolResult<Block> {
        log::debug!("consensus: send rpc pull block {}", number);
        let res = self
            .network
            .call::<BlockNumber, Block>(ctx, end, number, Priority::High)
            .await?;
        Ok(res)
    }

    #[trace_span(kind = "consensus.adapter", logs = "{txs_len: txs.len()}")]
    async fn verify_txs(&self, ctx: Context, number: u64, txs: &[Hash]) -> ProtocolResult<()> {
        if let Err(e) = self.mempool.ensure_order_txs(ctx, Some(number), txs).await {
            log::error!("verify_txs error {:?}", e);
            return Err(ConsensusError::VerifyTransaction(number).into());
        }

        Ok(())
    }
}

#[async_trait]
impl<M, N, S, DB> SynchronizationAdapter for OverlordConsensusAdapter<M, N, S, DB>
where
    M: MemPool + 'static,
    N: Network + Rpc + PeerTrust + Gossip + 'static,
    S: Storage + 'static,
    DB: trie::DB + 'static,
{
    #[trace_span(kind = "consensus.adapter")]
    fn update_status(
        &self,
        ctx: Context,
        number: u64,
        consensus_interval: u64,
        propose_ratio: u64,
        prevote_ratio: u64,
        precommit_ratio: u64,
        brake_ratio: u64,
        validators: Vec<Validator>,
    ) -> ProtocolResult<()> {
        self.overlord_handler
            .read()
            .as_ref()
            .expect("Please set the overlord handle first")
            .send_msg(
                ctx,
                OverlordMsg::RichStatus(gen_overlord_status(
                    number + 1,
                    consensus_interval,
                    propose_ratio,
                    prevote_ratio,
                    precommit_ratio,
                    brake_ratio,
                    validators,
                )),
            )
            .map_err(|e| ConsensusError::OverlordErr(Box::new(e)))?;
        Ok(())
    }

    /// Pull some blocks from other nodes from `begin` to `end`.
    #[trace_span(kind = "consensus.adapter")]
    async fn get_block_from_remote(&self, ctx: Context, number: u64) -> ProtocolResult<Block> {
        let res = self
            .network
            .call::<BlockNumber, Block>(ctx, RPC_SYNC_PULL_BLOCK, number, Priority::High)
            .await;
        match res {
            Ok(data) => {
                common_apm::metrics::consensus::CONSENSUS_RESULT_COUNTER_VEC_STATIC
                    .get_block_from_remote
                    .success
                    .inc();
                Ok(data)
            }
            Err(err) => {
                common_apm::metrics::consensus::CONSENSUS_RESULT_COUNTER_VEC_STATIC
                    .get_block_from_remote
                    .failure
                    .inc();
                Err(err)
            }
        }
    }

    /// Pull signed transactions corresponding to the given hashes from other
    /// nodes.
    #[trace_span(kind = "consensus.adapter", logs = "{txs_len: hashes.len()}")]
    async fn get_txs_from_remote(
        &self,
        ctx: Context,
        number: u64,
        hashes: &[Hash],
    ) -> ProtocolResult<Vec<SignedTransaction>> {
        let res = self
            .network
            .call::<PullTxsRequest, BatchSignedTxs>(
                ctx,
                RPC_SYNC_PULL_TXS,
                PullTxsRequest::new(number, hashes.to_vec()),
                Priority::High,
            )
            .await?;
        Ok(res.inner())
    }

    /// Pull a proof of certain block from other nodes
    #[trace_span(kind = "consensus.adapter")]
    async fn get_proof_from_remote(&self, ctx: Context, number: u64) -> ProtocolResult<Proof> {
        let ret = self
            .network
            .call::<BlockNumber, Proof>(ctx, RPC_SYNC_PULL_PROOF, number, Priority::High)
            .await?;
        Ok(ret)
    }

    fn get_tx_from_mem(&self, ctx: Context, tx_hash: &Hash) -> Option<SignedTransaction> {
        self.mempool.get_tx_from_mem(ctx, tx_hash)
    }
}

#[async_trait]
impl<M, N, S, DB> CommonConsensusAdapter for OverlordConsensusAdapter<M, N, S, DB>
where
    M: MemPool + 'static,
    N: Network + Rpc + PeerTrust + Gossip + 'static,
    S: Storage + 'static,
    DB: trie::DB + 'static,
{
    /// Save a block to the database.
    #[trace_span(kind = "consensus.adapter", logs = "{txs_len: block.tx_hashes.len()}")]
    async fn save_block(&self, ctx: Context, block: Block) -> ProtocolResult<()> {
        self.storage.insert_block(ctx, block).await
    }

    #[trace_span(kind = "consensus.adapter")]
    async fn save_proof(&self, ctx: Context, proof: Proof) -> ProtocolResult<()> {
        self.storage.update_latest_proof(ctx, proof).await
    }

    /// Save some signed transactions to the database.
    #[trace_span(kind = "consensus.adapter", logs = "{txs_len: signed_txs.len()}")]
    async fn save_signed_txs(
        &self,
        ctx: Context,
        block_number: u64,
        signed_txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<()> {
        self.storage
            .insert_transactions(ctx, block_number, signed_txs)
            .await
    }

    #[trace_span(kind = "consensus.adapter", logs = "{receipts_len: receipts.len()}")]
    async fn save_receipts(
        &self,
        ctx: Context,
        number: u64,
        receipts: Vec<Receipt>,
    ) -> ProtocolResult<()> {
        self.storage.insert_receipts(ctx, number, receipts).await
    }

    /// Flush the given transactions in the mempool.
    #[trace_span(
        kind = "consensus.adapter",
        logs = "{flush_txs_len: ordered_tx_hashes.len()}"
    )]
    async fn flush_mempool(
        &self,
        ctx: Context,
        ordered_tx_hashes: &[Hash],
        current_number: BlockNumber,
    ) -> ProtocolResult<()> {
        self.mempool
            .flush(ctx, ordered_tx_hashes, current_number)
            .await
    }

    /// Get a block corresponding to the given number.
    #[trace_span(kind = "consensus.adapter")]
    async fn get_block_by_number(&self, ctx: Context, number: u64) -> ProtocolResult<Block> {
        self.storage
            .get_block(ctx, number)
            .await?
            .ok_or_else(|| ConsensusError::StorageItemNotFound.into())
    }

    async fn get_block_header_by_number(
        &self,
        ctx: Context,
        number: u64,
    ) -> ProtocolResult<Header> {
        self.storage
            .get_block_header(ctx, number)
            .await?
            .ok_or_else(|| ConsensusError::StorageItemNotFound.into())
    }

    /// Get the current number from storage.
    #[trace_span(kind = "consensus.adapter")]
    async fn get_current_number(&self, ctx: Context) -> ProtocolResult<u64> {
        let header = self.storage.get_latest_block_header(ctx).await?;
        Ok(header.number)
    }

    #[trace_span(kind = "consensus.adapter", logs = "{txs_len: tx_hashes.len()}")]
    async fn get_txs_from_storage(
        &self,
        ctx: Context,
        tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<SignedTransaction>> {
        let futs = tx_hashes
            .iter()
            .map(|tx_hash| self.storage.get_transaction_by_hash(ctx.clone(), tx_hash))
            .collect::<Vec<_>>();
        futures::future::try_join_all(futs)
            .await
            .map(|txs| txs.into_iter().flatten().collect::<Vec<_>>())
    }

    #[allow(unused_braces)]
    #[trace_span(kind = "consensus.adapter")]
    async fn exec(
        &self,
        ctx: Context,
        last_state_root: Hash,
        proposal: &Proposal,
        signed_txs: &[SignedTransaction],
    ) -> ProtocolResult<ExecResp> {
        let mut backend = AxonExecutorAdapter::from_root(
            last_state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            proposal.clone().into(),
        )?;

        let verifier_list = self
            .metadata
            .get_metadata_by_block_number(proposal.number)?
            .verifier_list;

        Ok(task::block_in_place(|| {
            let time = Instant::now();
            let res = AxonExecutor::default().exec(&mut backend, signed_txs, &verifier_list);
            common_apm::metrics::consensus::CONSENSUS_TIME_HISTOGRAM_VEC_STATIC
                .exec
                .observe(common_apm::metrics::duration_to_sec(time.elapsed()));

            res
        }))
    }

    fn is_last_block_in_current_epoch(&self, block_number: u64) -> ProtocolResult<bool> {
        self.metadata.is_last_block_in_current_epoch(block_number)
    }

    fn get_metadata_by_block_number(&self, block_number: u64) -> ProtocolResult<Metadata> {
        self.metadata.get_metadata_by_block_number(block_number)
    }

    #[trace_span(kind = "consensus.adapter")]
    async fn broadcast_number(&self, ctx: Context, number: u64) -> ProtocolResult<()> {
        self.network
            .broadcast(ctx, BROADCAST_HEIGHT, number, Priority::High)
            .await
    }

    fn set_args(&self, context: Context, state_root: MerkleRoot, gas_limit: u64, max_tx_size: u64) {
        self.mempool
            .set_args(context, state_root, gas_limit, max_tx_size);
    }

    fn tag_consensus(&self, ctx: Context, pub_keys: Vec<Bytes>) -> ProtocolResult<()> {
        let peer_ids_bytes = pub_keys
            .iter()
            .map(|pk| PeerId::from_pubkey_bytes(pk).map(PeerIdExt::into_bytes_ext))
            .collect::<Result<_, _>>()?;

        self.network.tag_consensus(ctx, peer_ids_bytes)
    }

    /// this function verify all info in header except proof and roots
    #[trace_span(kind = "consensus.adapter")]
    async fn verify_block_header(&self, ctx: Context, proposal: &Proposal) -> ProtocolResult<()> {
        let previous_block = self
            .get_block_by_number(ctx.clone(), proposal.number - 1)
            .await
            .map_err(|e| {
                log::error!(
                    "[consensus] verify_block_header, previous_block_header {} fails",
                    proposal.number - 1,
                );
                e
            })?;

        let previous_block_hash = previous_block.hash();

        if previous_block_hash != proposal.prev_hash {
            log::error!(
                "[consensus] verify_block_header, previous_block_hash: {:?}, block.header.prev_hash: {:?}",
                previous_block_hash,
                proposal.prev_hash
            );
            return Err(
                ConsensusError::VerifyBlockHeader(proposal.number, PreviousBlockHash).into(),
            );
        }

        Ok(())
    }

    #[trace_span(kind = "consensus.adapter")]
    async fn verify_proof(&self, ctx: Context, block: Block, proof: Proof) -> ProtocolResult<()> {
        // the block 0 has no proof, which is consensus-ed by community, not by chain
        if block.header.number == 0 {
            return Ok(());
        };

        if block.header.number != proof.number {
            log::error!(
                "[consensus] verify_proof, block_header.number: {}, proof.number: {}",
                block.header.number,
                proof.number
            );
            return Err(ConsensusError::VerifyProof(
                block.header.number,
                HeightMismatch(block.header.number, proof.number),
            )
            .into());
        }

        let proposal_hash = Proposal::from(&block).hash();

        if proposal_hash != proof.block_hash {
            log::error!(
                "[consensus] verify_proof, blockhash: {:?}, proof.block_hash: {:?}",
                proposal_hash,
                proof.block_hash
            );
            return Err(ConsensusError::VerifyProof(block.header.number, HashMismatch).into());
        }

        // the auth_list for the target should comes from previous number
        let metadata = self
            .metadata
            .get_metadata_by_block_number(block.header.number)?;

        if !metadata.version.contains(block.header.number) {
            return Err(ConsensusError::ConfusedMetadata(
                metadata.version.start,
                metadata.version.end,
            )
            .into());
        }

        let mut authority_list = metadata
            .verifier_list
            .iter()
            .map(|v| Node {
                address:        v.pub_key.as_bytes(),
                propose_weight: v.propose_weight,
                vote_weight:    v.vote_weight,
            })
            .collect::<Vec<Node>>();

        let signed_voters = extract_voters(&mut authority_list, &proof.bitmap).map_err(|_| {
            log::error!("[consensus] extract_voters fails, bitmap error");
            ConsensusError::VerifyProof(block.header.number, BitMap)
        })?;

        let vote = Vote {
            height:     proof.number,
            round:      proof.round,
            vote_type:  VoteType::Precommit,
            block_hash: Bytes::from(proof.block_hash.as_bytes().to_vec()),
        };

        let weight_map = authority_list
            .iter()
            .map(|node| (node.address.clone(), node.vote_weight))
            .collect::<HashMap<overlord::types::Address, u32>>();
        self.verify_proof_weight(
            ctx.clone(),
            block.header.number,
            weight_map,
            signed_voters.clone(),
        )?;

        let vote_hash = self.crypto.hash(Bytes::from(rlp::encode(&vote)));
        let hex_pubkeys = metadata
            .verifier_list
            .iter()
            .filter_map(|v| {
                if signed_voters.contains(&v.pub_key.as_bytes()) {
                    Some(v.bls_pub_key.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        self.verify_proof_signature(
            ctx,
            block.header.number,
            vote_hash.clone(),
            proof.signature.clone(),
            hex_pubkeys,
        ).map_err(|e| {
            log::error!("[consensus] verify_proof_signature error, number {}, vote: {:?}, vote_hash:{:?}, sig:{:?}, signed_voter:{:?}",
            block.header.number,
            vote,
            vote_hash,
            proof.signature,
            signed_voters,
            );
            e
        })?;

        Ok(())
    }

    #[trace_span(kind = "consensus.adapter")]
    fn verify_proof_signature(
        &self,
        ctx: Context,
        block_number: u64,
        vote_hash: Bytes,
        aggregated_signature_bytes: Bytes,
        vote_keys: Vec<Hex>,
    ) -> ProtocolResult<()> {
        let pub_keys = vote_keys
            .into_iter()
            .map(convert_hex_to_bls_pubkeys)
            .collect::<Result<Vec<_>, _>>()?;

        self.crypto
            .inner_verify_aggregated_signature(vote_hash, pub_keys, aggregated_signature_bytes)
            .map_err(|e| {
                log::error!("[consensus] verify_proof_signature error: {}", e);
                ConsensusError::VerifyProof(block_number, Signature).into()
            })
    }

    #[trace_span(kind = "consensus.adapter")]
    fn verify_proof_weight(
        &self,
        ctx: Context,
        block_number: u64,
        weight_map: HashMap<Bytes, u32>,
        signed_voters: Vec<Bytes>,
    ) -> ProtocolResult<()> {
        let total_validator_weight: u64 = weight_map.iter().map(|pair| u64::from(*pair.1)).sum();

        let mut accumulator = 0u64;
        for signed_voter_address in signed_voters {
            if weight_map.contains_key(signed_voter_address.as_ref()) {
                let weight = weight_map
                    .get(signed_voter_address.as_ref())
                    .ok_or(ConsensusError::VerifyProof(block_number, WeightNotFound))
                    .map_err(|e| {
                        log::error!(
                            "[consensus] verify_proof_weight,signed_voter_address: {:?}",
                            signed_voter_address
                        );
                        e
                    })?;
                accumulator += u64::from(*(weight));
            } else {
                log::error!(
                    "[consensus] verify_proof_weight, weight not found, signed_voter_address: {:?}",
                    signed_voter_address
                );
                return Err(
                    ConsensusError::VerifyProof(block_number, BlockProofField::Validator).into(),
                );
            }
        }

        if 3 * accumulator <= 2 * total_validator_weight {
            log::error!(
                "[consensus] verify_proof_weight, accumulator: {}, total: {}",
                accumulator,
                total_validator_weight
            );

            return Err(ConsensusError::VerifyProof(block_number, BlockProofField::Weight).into());
        }
        Ok(())
    }
}

impl<M, N, S, DB> OverlordConsensusAdapter<M, N, S, DB>
where
    M: MemPool + 'static,
    N: Rpc + PeerTrust + Gossip + 'static,
    S: Storage + 'static,
    DB: trie::DB + 'static,
{
    pub fn new(
        network: Arc<N>,
        mempool: Arc<M>,
        storage: Arc<S>,
        trie_db: Arc<DB>,
        metadata: Arc<MetadataHandle>,
        crypto: Arc<OverlordCrypto>,
    ) -> ProtocolResult<Self> {
        Ok(OverlordConsensusAdapter {
            network,
            mempool,
            storage,
            metadata,
            trie_db,
            overlord_handler: RwLock::new(None),
            crypto,
        })
    }

    pub fn set_overlord_handler(&self, handler: OverlordHandler<Proposal>) {
        *self.overlord_handler.write() = Some(handler)
    }
}
