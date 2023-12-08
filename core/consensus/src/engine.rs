use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::error::Error;
use std::sync::Arc;

use json::JsonValue;
use log::{error, info};
use overlord::error::ConsensusError as OverlordError;
use overlord::types::{Commit, Node, OverlordMsg, Status, ViewChangeReason};
use overlord::{Consensus as Engine, Wal};
use parking_lot::RwLock;
use rlp::Encodable;

use common_apm::Instant;
use common_apm_derive::trace_span;
use common_crypto::BlsPublicKey;
use common_logger::{json, log};
use common_merkle::TrieMerkle;
use core_executor::MetadataHandle;
use protocol::constants::endpoints::{
    END_GOSSIP_AGGREGATED_VOTE, END_GOSSIP_SIGNED_CHOKE, END_GOSSIP_SIGNED_PROPOSAL,
    END_GOSSIP_SIGNED_VOTE,
};
use protocol::constants::{BASE_FEE_PER_GAS, MAX_BLOCK_GAS_LIMIT};
use protocol::traits::{ConsensusAdapter, Context, MessageTarget, NodeInfo};
use protocol::types::{
    Block, BlockVersion, Bytes, ExecResp, ExtraData, Hash, Hex, Metadata, Proof, Proposal,
    SignedTransaction, ValidatorExtend, VecDisplayHelper, RLP_NULL,
};
use protocol::{
    async_trait, codec::ProtocolCodec, tokio::sync::Mutex as AsyncMutex, types::HardforkInfoInner,
    ProtocolError, ProtocolResult,
};

use crate::status::{CurrentStatus, StatusAgent};
use crate::stop_signal::StopSignal;
use crate::util::{digest_signed_transactions, time_now, OverlordCrypto};
use crate::wal::{ConsensusWal, SignedTxsWAL};
use crate::ConsensusError;

/// validator is for create new block, and authority is for build overlord
/// status.
pub struct ConsensusEngine<Adapter> {
    status:         StatusAgent,
    node_info:      NodeInfo,
    exemption_hash: RwLock<HashSet<Hash>>,

    adapter: Arc<Adapter>,
    txs_wal: Arc<SignedTxsWAL>,
    crypto:  Arc<OverlordCrypto>,
    lock:    Arc<AsyncMutex<()>>,

    last_commit_time:             RwLock<u64>,
    consensus_wal:                Arc<ConsensusWal>,
    last_check_block_fail_reason: RwLock<String>,

    stop_signal: StopSignal,
}

#[async_trait]
impl<Adapter: ConsensusAdapter + 'static> Engine<Proposal> for ConsensusEngine<Adapter> {
    #[trace_span(kind = "consensus.engine", logs = "{next_number: next_number}")]
    async fn get_block(
        &self,
        ctx: Context,
        next_number: u64,
    ) -> Result<(Proposal, Bytes), Box<dyn Error + Send>> {
        let status = self.status.inner();
        let txs = self
            .adapter
            .get_txs_from_mempool(
                ctx.clone(),
                next_number,
                MAX_BLOCK_GAS_LIMIT.into(),
                status.tx_num_limit,
            )
            .await?;
        let signed_txs = self.adapter.get_full_txs(ctx.clone(), &txs.hashes).await?;
        let txs_root = if !txs.hashes.is_empty() {
            TrieMerkle::from_iter(txs.hashes.iter().enumerate())
                .root_hash()
                .unwrap_or_else(|err| {
                    panic!("failed to calculate trie root hash for transactions since {err}")
                })
        } else {
            RLP_NULL
        };

        let mut remove = false;
        let extra_data_hardfork = {
            let mut hardfork = self.node_info.hardfork_proposals.write().unwrap();
            match &*hardfork {
                Some(v) => {
                    // remove invalid proposal if the proposed block height is passed
                    if v.block_number <= next_number {
                        hardfork.take();
                        remove = true;
                        Vec::new()
                    } else {
                        vec![ExtraData {
                            inner: v.encode().unwrap(),
                        }]
                    }
                }
                None => Vec::new(),
            }
        };

        if remove {
            self.adapter.remove_hardfork_proposal(ctx.clone()).await?;
        }

        let proposal = Proposal {
            version:                  BlockVersion::V0,
            prev_hash:                status.prev_hash,
            proposer:                 self.node_info.self_address.0,
            prev_state_root:          self.status.inner().last_state_root,
            transactions_root:        txs_root,
            signed_txs_hash:          digest_signed_transactions(&signed_txs),
            timestamp:                time_now(),
            number:                   next_number,
            gas_limit:                MAX_BLOCK_GAS_LIMIT.into(),
            extra_data:               extra_data_hardfork,
            base_fee_per_gas:         BASE_FEE_PER_GAS.into(),
            proof:                    status.proof,
            chain_id:                 self.node_info.chain_id,
            call_system_script_count: txs.call_system_script_count,
            tx_hashes:                txs.hashes,
        };

        if proposal.number != proposal.proof.number + 1 {
            return Err(ProtocolError::from(ConsensusError::InvalidProof {
                expect: proposal.number - 1,
                actual: proposal.proof.number,
            })
            .into());
        }

        let hash = proposal.hash();
        let mut set = self.exemption_hash.write();
        set.insert(hash);

        Ok((proposal, Bytes::from(hash.as_bytes().to_vec())))
    }

    #[trace_span(
        kind = "consensus.engine",
        logs = "{
            next_number: next_number,
            hash: Hex::encode(hash.clone()).as_string(),
            txs_len: proposal.tx_hashes.len()}"
    )]
    async fn check_block(
        &self,
        ctx: Context,
        next_number: u64,
        hash: Bytes,
        proposal: Proposal,
    ) -> Result<(), Box<dyn Error + Send>> {
        let time = Instant::now();

        if proposal.number != proposal.proof.number + 1 {
            return Err(ProtocolError::from(ConsensusError::InvalidProof {
                expect: proposal.number - 1,
                actual: proposal.proof.number,
            })
            .into());
        }

        if let Some(t) = proposal.extra_data.get(0) {
            match HardforkInfoInner::decode(&t.inner) {
                Ok(data) => {
                    if !self
                        .node_info
                        .hardfork_proposals
                        .read()
                        .unwrap()
                        .as_ref()
                        .map(|v| &data == v)
                        .unwrap_or_default()
                    {
                        return Err(ProtocolError::from(ConsensusError::Hardfork(
                            "hardfork proposal doesn't match".to_string(),
                        ))
                        .into());
                    }
                }
                Err(_) => {
                    return Err(ProtocolError::from(ConsensusError::Hardfork(
                        "hardfork proposal can't decode".to_string(),
                    ))
                    .into())
                }
            }
        }

        let tx_hashes = proposal.tx_hashes.clone();
        let tx_hashes_len = tx_hashes.len();
        let exemption = {
            self.exemption_hash
                .read()
                .contains(&Hash::from_slice(hash.as_ref()))
        };

        gauge_txs_len(&proposal);

        // If the block is proposed by self, it does not need to check. Get full signed
        // transactions directly.
        if !exemption {
            if let Err(e) = self.inner_check_block(ctx.clone(), &proposal).await {
                let mut reason = self.last_check_block_fail_reason.write();
                *reason = e.to_string();
                return Err(e.into());
            }
        }

        common_apm::metrics::consensus::CONSENSUS_CHECK_BLOCK_HISTOGRAM_VEC_STATIC
            .check_block_cost
            .observe(common_apm::metrics::duration_to_sec(time.elapsed()));
        info!("[consensus-engine]: check block cost {:?}", time.elapsed());

        let time = Instant::now();
        let txs = self.adapter.get_full_txs(ctx, &tx_hashes).await?;
        common_apm::metrics::consensus::CONSENSUS_CHECK_BLOCK_HISTOGRAM_VEC_STATIC
            .get_txs_cost
            .observe(common_apm::metrics::duration_to_sec(time.elapsed()));
        info!("[consensus-engine]: get txs cost {:?}", time.elapsed());

        let time = Instant::now();
        self.txs_wal
            .save(next_number, proposal.transactions_root, txs)?;
        common_apm::metrics::consensus::CONSENSUS_CHECK_BLOCK_HISTOGRAM_VEC_STATIC
            .write_wal_cost
            .observe(common_apm::metrics::duration_to_sec(time.elapsed()));
        info!(
            "[consensus-engine]: write wal cost {:?} tx_hashes_len {}",
            time.elapsed(),
            tx_hashes_len
        );

        Ok(())
    }

    /// **TODO:** the overlord interface and process needs to be changed.
    /// Get the `FixedSignedTxs` from the argument rather than get it from
    /// mempool.
    #[trace_span(
        kind = "consensus.engine",
        logs = "{
            current_number: current_number,
            txs_len: commit.content.tx_hashes.len()}"
    )]
    async fn commit(
        &self,
        ctx: Context,
        current_number: u64,
        commit: Commit<Proposal>,
    ) -> Result<Status, Box<dyn Error + Send>> {
        self.stop_signal
            .check_height_and_send(current_number.saturating_sub(1));
        if self.stop_signal.is_stopped() {
            return Err(
                ProtocolError::from(ConsensusError::Other("node is shutdown".to_string())).into(),
            );
        }

        let lock = self.lock.try_lock();
        if lock.is_err() {
            return Err(ProtocolError::from(ConsensusError::LockInSync).into());
        }

        let status = self.status.inner();
        let metadata = self
            .adapter
            .get_metadata_by_block_number(current_number + 1)
            .await?;

        if current_number == status.last_number {
            return Ok(Status {
                height:         current_number + 1,
                interval:       Some(metadata.consensus_config.interval),
                authority_list: convert_to_overlord_authority(&metadata.verifier_list),
                timer_config:   Some(metadata.into()),
            });
        }

        if current_number != status.last_number + 1 {
            return Err(ProtocolError::from(ConsensusError::OutdatedCommit(
                current_number,
                status.last_number,
            ))
            .into());
        }

        let proposal = commit.content;
        let txs_len = proposal.tx_hashes.len();

        // Storage save the latest proof.
        let proof = Proof {
            number:     commit.proof.height,
            round:      commit.proof.round,
            block_hash: Hash::from_slice(commit.proof.block_hash.as_ref()),
            signature:  commit.proof.signature.signature.clone(),
            bitmap:     commit.proof.signature.address_bitmap.clone(),
        };

        // Get full transactions from mempool. If is error, try get from wal.
        let signed_txs = match self
            .adapter
            .get_full_txs(ctx.clone(), &proposal.tx_hashes)
            .await
        {
            Ok(txs) => txs,
            Err(_) => self
                .txs_wal
                .load(current_number, proposal.transactions_root)?,
        };

        // Execute transactions
        let resp = self
            .adapter
            .exec(
                ctx.clone(),
                self.status.inner().last_state_root,
                &proposal,
                &signed_txs,
            )
            .await?;

        info!(
            "[consensus]: validator of number {} is {}",
            current_number + 1,
            VecDisplayHelper(&metadata.verifier_list[..])
        );

        self.update_status(ctx.clone(), resp, proposal.clone(), proof, signed_txs)
            .await?;

        self.adapter
            .flush_mempool(ctx.clone(), &proposal.tx_hashes, current_number)
            .await?;

        self.txs_wal.remove(current_number.saturating_sub(2))?;

        {
            self.exemption_hash.write().clear();
        }

        let next_block_number = current_number + 1;
        let metadata = self
            .adapter
            .get_metadata_by_block_number(next_block_number)
            .await?;
        let epoch = metadata.epoch;
        let status = Status {
            height:         next_block_number,
            interval:       Some(metadata.consensus_config.interval),
            authority_list: convert_to_overlord_authority(&metadata.verifier_list),
            timer_config:   Some(metadata.into()),
        };

        self.adapter.broadcast_number(ctx, current_number).await?;
        self.alert_missing_next_metadata(epoch).await;
        self.metric_commit(current_number, txs_len);

        Ok(status)
    }

    /// Only signed proposal and aggregated vote will be broadcast to others.
    #[trace_span(kind = "consensus.engine")]
    async fn broadcast_to_other(
        &self,
        ctx: Context,
        msg: OverlordMsg<Proposal>,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (end, msg) = match msg {
            OverlordMsg::SignedProposal(sp) => {
                let bytes = sp.rlp_bytes();
                (END_GOSSIP_SIGNED_PROPOSAL, bytes)
            }

            OverlordMsg::AggregatedVote(av) => {
                let bytes = av.rlp_bytes();
                (END_GOSSIP_AGGREGATED_VOTE, bytes)
            }

            OverlordMsg::SignedChoke(sc) => {
                let bytes = sc.rlp_bytes();
                (END_GOSSIP_SIGNED_CHOKE, bytes)
            }

            _ => unreachable!(),
        };

        self.adapter
            .transmit(ctx, msg.freeze().to_vec(), end, MessageTarget::Broadcast)
            .await?;
        Ok(())
    }

    /// Only signed vote will be transmit to the relayer.
    #[trace_span(
        kind = "consensus.engine",
        logs = "{pub_key: Hex::encode(pub_key.clone()).as_string()}"
    )]
    async fn transmit_to_relayer(
        &self,
        ctx: Context,
        pub_key: Bytes,
        msg: OverlordMsg<Proposal>,
    ) -> Result<(), Box<dyn Error + Send>> {
        match msg {
            OverlordMsg::SignedVote(sv) => {
                let msg = sv.rlp_bytes();
                self.adapter
                    .transmit(
                        ctx,
                        msg.freeze().to_vec(),
                        END_GOSSIP_SIGNED_VOTE,
                        MessageTarget::Specified(pub_key),
                    )
                    .await?;
            }
            OverlordMsg::AggregatedVote(av) => {
                let msg = av.rlp_bytes();
                self.adapter
                    .transmit(
                        ctx,
                        msg.freeze().to_vec(),
                        END_GOSSIP_AGGREGATED_VOTE,
                        MessageTarget::Specified(pub_key),
                    )
                    .await?;
            }
            _ => unreachable!(),
        };
        Ok(())
    }

    /// This function is rarely used, so get the authority list from the
    /// RocksDB.
    #[trace_span(kind = "consensus.engine", logs = "{next_number: next_number}")]
    async fn get_authority_list(
        &self,
        ctx: Context,
        next_number: u64,
    ) -> Result<Vec<Node>, Box<dyn Error + Send>> {
        if next_number == 0 {
            return Ok(vec![]);
        }

        let current_metadata = self
            .adapter
            .get_metadata_by_block_number(next_number)
            .await?;
        let old_metadata = if current_metadata.version.contains(next_number - 1) {
            current_metadata
        } else {
            self.adapter
                .get_metadata_by_block_number(next_number - 1)
                .await?
        };

        // The address field of Node struct should use the node's secp256k1 public key
        let mut old_validators = old_metadata
            .verifier_list
            .into_iter()
            .map(|v| Node {
                address:        v.pub_key.as_bytes(),
                propose_weight: v.propose_weight,
                vote_weight:    v.vote_weight,
            })
            .collect::<Vec<_>>();
        old_validators.sort();
        Ok(old_validators)
    }

    fn report_error(&self, _ctx: Context, _err: OverlordError) {}

    fn report_view_change(&self, cx: Context, number: u64, round: u64, reason: ViewChangeReason) {
        let view_change_reason = match reason {
            ViewChangeReason::CheckBlockNotPass => {
                let e = self.last_check_block_fail_reason.read();
                reason.to_string() + " " + e.as_str()
            }
            _ => reason.to_string(),
        };

        log(
            log::Level::Warn,
            "consensus",
            "cons000",
            &cx,
            json!({"number", number; "round", round; "reason", view_change_reason}),
        );
    }
}

#[async_trait]
impl<Adapter: ConsensusAdapter + 'static> Wal for ConsensusEngine<Adapter> {
    async fn save(&self, info: Bytes) -> Result<(), Box<dyn Error + Send>> {
        self.consensus_wal
            .update_overlord_wal(Context::new(), info)
            .map_err(|e| ProtocolError::from(ConsensusError::Other(e.to_string())))?;
        Ok(())
    }

    async fn load(&self) -> Result<Option<Bytes>, Box<dyn Error + Send>> {
        let res = self.consensus_wal.load_overlord_wal(Context::new()).ok();
        Ok(res)
    }
}

impl<Adapter: ConsensusAdapter + 'static> ConsensusEngine<Adapter> {
    pub fn new(
        status: StatusAgent,
        node_info: NodeInfo,
        wal: Arc<SignedTxsWAL>,
        adapter: Arc<Adapter>,
        crypto: Arc<OverlordCrypto>,
        lock: Arc<AsyncMutex<()>>,
        consensus_wal: Arc<ConsensusWal>,
        stop_signal: StopSignal,
    ) -> Self {
        Self {
            status,
            node_info,
            exemption_hash: RwLock::new(HashSet::new()),
            txs_wal: wal,
            adapter,
            crypto,
            lock,
            last_commit_time: RwLock::new(time_now()),
            consensus_wal,
            last_check_block_fail_reason: RwLock::new(String::new()),
            stop_signal,
        }
    }

    pub fn status(&self) -> CurrentStatus {
        self.status.inner()
    }

    async fn inner_check_block(&self, ctx: Context, proposal: &Proposal) -> ProtocolResult<()> {
        let current_timestamp = time_now();

        self.adapter
            .verify_block_header(ctx.clone(), proposal)
            .await
            .map_err(|e| {
                error!(
                    "[consensus] check_block, verify_block_header error, proposal: {}",
                    proposal
                );
                e
            })?;

        // verify the proof in the block for previous block
        // skip to get previous proof to compare because the node may just comes from
        // sync and waste a delay of read
        let previous_block = self
            .adapter
            .get_block_by_number(ctx.clone(), proposal.number - 1)
            .await?;

        // verify block timestamp.
        if !validate_timestamp(
            current_timestamp,
            proposal.timestamp,
            previous_block.header.timestamp,
        ) {
            return Err(ProtocolError::from(ConsensusError::InvalidTimestamp));
        }

        self.adapter
            .verify_proof(
                ctx.clone(),
                previous_block.clone(),
                proposal.proof.clone(),
            )
            .await
            .map_err(|e| {
                error!(
                    "[consensus] check_block, verify_proof error, previous block header: {}, proof: {}",
                    previous_block.header,
                    proposal.proof
                );
                e
            })?;

        self.adapter
            .verify_txs(ctx.clone(), proposal.number, &proposal.tx_hashes)
            .await
            .map_err(|e| {
                error!("[consensus] check_block, verify_txs error",);
                e
            })?;

        let signed_txs = self
            .adapter
            .get_full_txs(ctx.clone(), &proposal.tx_hashes)
            .await?;
        self.check_order_transactions(ctx.clone(), proposal, &signed_txs)
    }

    #[trace_span(kind = "consensus.engine", logs = "{txs_len: signed_txs.len()}")]
    fn check_order_transactions(
        &self,
        ctx: Context,
        proposal: &Proposal,
        signed_txs: &[SignedTransaction],
    ) -> ProtocolResult<()> {
        let txs_root = if proposal.tx_hashes.is_empty() {
            RLP_NULL
        } else {
            TrieMerkle::from_iter(proposal.tx_hashes.iter().enumerate())
                .root_hash()
                .unwrap_or_else(|err| {
                    panic!("failed to calculate trie root hash for proposals since {err}")
                })
        };

        let stxs_hash = digest_signed_transactions(signed_txs);

        if stxs_hash != proposal.signed_txs_hash {
            return Err(ConsensusError::InvalidOrderSignedTransactionsHash {
                expect: stxs_hash,
                actual: proposal.signed_txs_hash,
            }
            .into());
        }

        if txs_root != proposal.transactions_root {
            return Err(ConsensusError::InvalidOrderRoot {
                expect: txs_root,
                actual: proposal.transactions_root,
            }
            .into());
        }

        Ok(())
    }

    /// After get the signed transactions:
    /// 1. Execute the signed transactions.
    /// 2. Save the signed transactions.
    /// 3. Save the latest proof.
    /// 4. Save the new block.
    /// 5. Save the receipt.
    pub async fn update_status(
        &self,
        ctx: Context,
        resp: ExecResp,
        proposal: Proposal,
        proof: Proof,
        txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<()> {
        let block = Block::new(proposal, resp.clone());
        let block_number = block.header.number;
        let block_hash = block.hash();

        let (receipts, _logs) = block.generate_receipts_and_logs(&txs, &resp);

        common_apm::metrics::consensus::ENGINE_ROUND_GAUGE.set(proof.round as i64);

        self.adapter.save_proof(ctx.clone(), proof.clone()).await?;

        // Save signed transactions
        self.adapter
            .save_signed_txs(ctx.clone(), block_number, txs)
            .await?;

        // Save transaction receipts
        self.adapter
            .save_receipts(ctx.clone(), block_number, receipts)
            .await?;

        // Save the block
        self.adapter.save_block(ctx.clone(), block.clone()).await?;

        let root = self
            .adapter
            .get_metadata_root(
                block.header.state_root,
                &Proposal::new_without_state_root(&block.header),
            )
            .await?;
        let handle = MetadataHandle::new(root);
        let hardforks = handle.hardfork_infos()?;

        let mut remove = false;
        if let Some(data) = hardforks.inner.last() {
            let mut self_proposal = self.node_info.hardfork_proposals.write().unwrap();

            if self_proposal
                .as_ref()
                .map(|v| data.flags & v.flags == v.flags)
                .unwrap_or_default()
            {
                // remove self hardfork proposal
                self_proposal.take();
                remove = true;
            }
        }

        if remove {
            self.adapter.remove_hardfork_proposal(ctx.clone()).await?;
        }

        if self
            .adapter
            .is_last_block_in_current_epoch(block_number)
            .await?
        {
            let metadata = self
                .adapter
                .get_metadata_by_block_number(block_number + 1)
                .await?;
            let pub_keys = metadata
                .verifier_list
                .iter()
                .map(|v| v.pub_key.as_bytes())
                .collect();
            self.adapter.tag_consensus(ctx.clone(), pub_keys)?;
            self.update_overlord_crypto(metadata)?;
        }

        let last_status = self.status.inner();
        let new_status = CurrentStatus {
            prev_hash:       block_hash,
            last_number:     block_number,
            last_state_root: resp.state_root,
            max_tx_size:     last_status.max_tx_size,
            tx_num_limit:    last_status.tx_num_limit,
            proof:           proof.clone(),
        };

        self.status.swap(new_status);

        // update timeout_gap of mempool
        self.adapter.set_args(
            ctx,
            resp.state_root,
            MAX_BLOCK_GAS_LIMIT,
            last_status.max_tx_size.low_u64(),
        );

        if block.header.number != proof.number {
            log::error!("[consensus] update_status for handle_commit error, before update, block number {}, proof number {}, proof {}",
                block_number,
                proof.number,
                proof
            );
        }

        Ok(())
    }

    fn update_overlord_crypto(&self, metadata: Metadata) -> ProtocolResult<()> {
        self.crypto.update(generate_new_crypto_map(metadata)?);
        Ok(())
    }

    async fn alert_missing_next_metadata(&self, current_epoch: u64) {
        let next_epoch = current_epoch + 1;
        if self
            .adapter
            .get_metadata_by_block_number(next_epoch)
            .await
            .is_err()
        {
            log::error!("Missing next {} metadata!", next_epoch);
        }
    }

    fn metric_commit(&self, current_height: u64, txs_len: usize) {
        common_apm::metrics::consensus::ENGINE_HEIGHT_GAUGE.set((current_height + 1) as i64);
        common_apm::metrics::consensus::ENGINE_COMMITTED_TX_COUNTER.inc_by(txs_len as u64);

        let now = time_now();
        let last_commit_time = *(self.last_commit_time.read());
        let elapsed = (now.saturating_sub(last_commit_time)) as f64;
        common_apm::metrics::consensus::ENGINE_CONSENSUS_COST_TIME.observe(elapsed / 1e3);
        let mut last_commit_time = self.last_commit_time.write();
        *last_commit_time = now;
    }
}

pub fn generate_new_crypto_map(metadata: Metadata) -> ProtocolResult<HashMap<Bytes, BlsPublicKey>> {
    let mut new_addr_pubkey_map = HashMap::new();
    for validator in metadata.verifier_list.into_iter() {
        let addr = validator.pub_key.as_bytes();
        let bls_pubkey = validator.bls_pub_key.as_bytes();
        let pubkey = BlsPublicKey::try_from(bls_pubkey.as_ref())
            .map_err(|err| ConsensusError::Other(format!("try from bls pubkey error {:?}", err)))?;
        new_addr_pubkey_map.insert(addr, pubkey);
    }
    Ok(new_addr_pubkey_map)
}

fn convert_to_overlord_authority(validators: &[ValidatorExtend]) -> Vec<Node> {
    // The address field of Node struct should use the node's secp256k1 public key
    let mut authority = validators
        .iter()
        .map(|v| Node {
            address:        v.pub_key.as_bytes(),
            propose_weight: v.propose_weight,
            vote_weight:    v.vote_weight,
        })
        .collect::<Vec<_>>();
    authority.sort();
    authority
}

fn validate_timestamp(
    current_timestamp: u64,
    proposal_timestamp: u64,
    previous_timestamp: u64,
) -> bool {
    if proposal_timestamp < previous_timestamp {
        log::error!(
            "[consensus] invalid timestamp previous {}, proposal {}",
            previous_timestamp,
            proposal_timestamp
        );
        return false;
    }

    if proposal_timestamp > current_timestamp {
        log::error!(
            "[consensus] invalid timestamp proposal {}, current {}",
            proposal_timestamp,
            current_timestamp
        );
        return false;
    }

    true
}

fn gauge_txs_len(proposal: &Proposal) {
    common_apm::metrics::consensus::ENGINE_ORDER_TX_GAUGE.set(proposal.tx_hashes.len() as i64);
}

#[cfg(test)]
mod tests {
    use super::validate_timestamp;

    #[test]
    fn test_validate_timestamp() {
        // current 10, proposal 9, previous 8. true
        assert!(validate_timestamp(10, 9, 8));

        // current 10, proposal 11, previous 8. true
        assert!(!validate_timestamp(10, 11, 8));

        // current 10, proposal 9, previous 11. true
        assert!(!validate_timestamp(10, 9, 11));
    }
}
