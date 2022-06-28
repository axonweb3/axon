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
use common_merkle::Merkle;
use protocol::codec::ProtocolCodec;
use protocol::traits::{ConsensusAdapter, Context, MessageTarget, NodeInfo};
use protocol::types::{
    Block, Bloom, BloomInput, Bytes, ExecResp, Hash, Hasher, Hex, Log, MerkleRoot, Metadata, Proof,
    Proposal, Receipt, SignedTransaction, TransactionAction, ValidatorExtend, BASE_FEE_PER_GAS,
    H160, MAX_BLOCK_GAS_LIMIT, U256,
};
use protocol::{
    async_trait, lazy::CURRENT_STATE_ROOT, tokio::sync::Mutex as AsyncMutex, ProtocolError,
    ProtocolResult,
};

use crate::message::{
    END_GOSSIP_AGGREGATED_VOTE, END_GOSSIP_SIGNED_CHOKE, END_GOSSIP_SIGNED_PROPOSAL,
    END_GOSSIP_SIGNED_VOTE,
};
use crate::status::{CurrentStatus, StatusAgent};
use crate::util::{digest_signed_transactions, time_now, OverlordCrypto};
use crate::wal::{ConsensusWal, SignedTxsWAL};
use crate::ConsensusError;

/// validator is for create new block, and authority is for build overlord
/// status.
pub struct ConsensusEngine<Adapter> {
    status:           StatusAgent,
    node_info:        NodeInfo,
    metadata_address: H160,
    exemption_hash:   RwLock<HashSet<Hash>>,

    adapter: Arc<Adapter>,
    txs_wal: Arc<SignedTxsWAL>,
    crypto:  Arc<OverlordCrypto>,
    lock:    Arc<AsyncMutex<()>>,

    cross_period_interval:        u64,
    last_commit_time:             RwLock<u64>,
    consensus_wal:                Arc<ConsensusWal>,
    last_check_block_fail_reason: RwLock<String>,
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
        let signed_txs = self.adapter.get_full_txs(ctx.clone(), &txs).await?;
        let order_root = Merkle::from_hashes(txs.clone())
            .get_root_hash()
            .unwrap_or_default();

        let proposal = Proposal {
            prev_hash:                  status.prev_hash,
            proposer:                   self.node_info.self_address.0,
            transactions_root:          order_root,
            signed_txs_hash:            digest_signed_transactions(&signed_txs),
            timestamp:                  time_now(),
            number:                     next_number,
            gas_limit:                  MAX_BLOCK_GAS_LIMIT.into(),
            extra_data:                 Default::default(),
            mixed_hash:                 None,
            base_fee_per_gas:           BASE_FEE_PER_GAS.into(),
            proof:                      status.proof,
            last_checkpoint_block_hash: status.last_checkpoint_block_hash,
            chain_id:                   self.node_info.chain_id,
            tx_hashes:                  txs,
        };

        if proposal.number != proposal.proof.number + 1 {
            return Err(ProtocolError::from(ConsensusError::InvalidProof {
                expect: proposal.number - 1,
                actual: proposal.proof.number,
            })
            .into());
        }

        let hash = Hasher::digest(proposal.encode()?);
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
            "[consensus-engine]: write wal cost {:?} tx_hashes_len {:?}",
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
        let lock = self.lock.try_lock();
        if lock.is_err() {
            return Err(ProtocolError::from(ConsensusError::LockInSync).into());
        }

        let status = self.status.inner();
        let metadata = self
            .adapter
            .get_metadata_unchecked(ctx.clone(), current_number + 1);

        if current_number == status.last_number {
            return Ok(Status {
                height:         current_number + 1,
                interval:       Some(metadata.interval),
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
        common_apm::metrics::consensus::ENGINE_ROUND_GAUGE.set(commit.proof.round as i64);

        self.adapter.save_proof(ctx.clone(), proof.clone()).await?;

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
                signed_txs.clone(),
            )
            .await?;

        info!(
            "[consensus]: validator of number {} is {:?}",
            current_number + 1,
            metadata.verifier_list
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
            .get_metadata_unchecked(ctx.clone(), next_block_number);
        let status = Status {
            height:         next_block_number,
            interval:       Some(metadata.interval),
            authority_list: convert_to_overlord_authority(&metadata.verifier_list),
            timer_config:   Some(metadata.into()),
        };

        self.adapter.broadcast_number(ctx, current_number).await?;

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
            .get_metadata_unchecked(ctx.clone(), next_number);
        let old_metadata = if current_metadata.version.contains(next_number - 1) {
            current_metadata
        } else {
            self.adapter.get_metadata_unchecked(ctx, next_number - 1)
        };

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
        metadata_address: H160,
        node_info: NodeInfo,
        wal: Arc<SignedTxsWAL>,
        adapter: Arc<Adapter>,
        crypto: Arc<OverlordCrypto>,
        lock: Arc<AsyncMutex<()>>,
        consensus_wal: Arc<ConsensusWal>,
        cross_period_interval: u64,
    ) -> Self {
        Self {
            status,
            metadata_address,
            node_info,
            exemption_hash: RwLock::new(HashSet::new()),
            txs_wal: wal,
            adapter,
            crypto,
            lock,
            cross_period_interval,
            last_commit_time: RwLock::new(time_now()),
            consensus_wal,
            last_check_block_fail_reason: RwLock::new(String::new()),
        }
    }

    pub fn status(&self) -> CurrentStatus {
        self.status.inner()
    }

    fn contains_change_metadata(&self, txs: &[SignedTransaction]) -> bool {
        let action = TransactionAction::Call(self.metadata_address);
        txs.iter()
            .any(|tx| tx.transaction.unsigned.action() == action)
    }

    async fn inner_check_block(&self, ctx: Context, proposal: &Proposal) -> ProtocolResult<()> {
        let current_timestamp = time_now();

        self.adapter
            .verify_block_header(ctx.clone(), proposal)
            .await
            .map_err(|e| {
                error!(
                    "[consensus] check_block, verify_block_header error, proposal: {:?}",
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
                    "[consensus] check_block, verify_proof error, previous block header: {:?}, proof: {:?}",
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
        let order_root = Merkle::from_hashes(proposal.tx_hashes.clone())
            .get_root_hash()
            .unwrap_or_default();

        let stxs_hash = Hasher::digest(rlp::encode_list(signed_txs));

        if stxs_hash != proposal.signed_txs_hash {
            return Err(ConsensusError::InvalidOrderSignedTransactionsHash {
                expect: stxs_hash,
                actual: proposal.signed_txs_hash,
            }
            .into());
        }

        if order_root != proposal.transactions_root {
            return Err(ConsensusError::InvalidOrderRoot {
                expect: order_root,
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
        let block_hash = block.header_hash();
        let is_change_metadata = self.contains_change_metadata(&txs);
        let next_block_number = block_number + 1;

        let (receipts, logs) = generate_receipts_and_logs(
            block_number,
            block_hash,
            block.header.state_root,
            &txs,
            &resp,
        );

        // Call cross client
        self.adapter
            .notify_block_logs(ctx.clone(), block_number, block_hash, &logs)
            .await;

        // Submit checkpoint
        if block_number % self.cross_period_interval == 0 {
            self.adapter
                .notify_checkpoint(ctx.clone(), block.clone(), proof.clone())
                .await;
        }

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

        if is_change_metadata {
            self.adapter.update_metadata(ctx.clone(), &block.header)?;
        }

        if self.adapter.need_change_metadata(next_block_number) {
            let metadata = self
                .adapter
                .get_metadata_unchecked(ctx.clone(), next_block_number);
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
            prev_hash:                  block_hash,
            last_number:                block_number,
            last_state_root:            resp.state_root,
            max_tx_size:                last_status.max_tx_size,
            tx_num_limit:               last_status.tx_num_limit,
            proof:                      proof.clone(),
            last_checkpoint_block_hash: last_status.last_checkpoint_block_hash,
        };

        CURRENT_STATE_ROOT.swap(Arc::new(resp.state_root));
        self.status.swap(new_status);

        // update timeout_gap of mempool
        self.adapter.set_args(
            ctx,
            resp.state_root,
            MAX_BLOCK_GAS_LIMIT,
            last_status.max_tx_size.as_u64(),
        );

        if block.header.number != proof.number {
            info!("[consensus] update_status for handle_commit, error, before update, block number {}, proof number:{}, proof : {:?}",
            block.header.number,
            proof.number,
            proof.clone());
        }

        Ok(())
    }

    fn update_overlord_crypto(&self, metadata: Metadata) -> ProtocolResult<()> {
        self.crypto.update(generate_new_crypto_map(metadata)?);
        Ok(())
    }

    fn metric_commit(&self, current_height: u64, txs_len: usize) {
        common_apm::metrics::consensus::ENGINE_HEIGHT_GAUGE.set((current_height + 1) as i64);
        common_apm::metrics::consensus::ENGINE_COMMITED_TX_COUNTER.inc_by(txs_len as u64);

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
        let hex_pubkey = validator.bls_pub_key.as_bytes();
        let pubkey = BlsPublicKey::try_from(hex_pubkey.as_ref())
            .map_err(|err| ConsensusError::Other(format!("try from bls pubkey error {:?}", err)))?;
        new_addr_pubkey_map.insert(addr, pubkey);
    }
    Ok(new_addr_pubkey_map)
}

fn convert_to_overlord_authority(validators: &[ValidatorExtend]) -> Vec<Node> {
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
            "[consensus] invalid timestamp previous {:?}, proposal {:?}",
            previous_timestamp,
            proposal_timestamp
        );
        return false;
    }

    if proposal_timestamp > current_timestamp {
        log::error!(
            "[consensus] invalid timestamp proposal {:?}, current {:?}",
            proposal_timestamp,
            current_timestamp
        );
        return false;
    }

    true
}

pub fn generate_receipts_and_logs(
    block_number: u64,
    block_hash: Hash,
    state_root: MerkleRoot,
    txs: &[SignedTransaction],
    resp: &ExecResp,
) -> (Vec<Receipt>, Vec<Vec<Log>>) {
    let mut log_index = 0;
    let receipts = txs
        .iter()
        .enumerate()
        .zip(resp.tx_resp.iter())
        .map(|((idx, tx), res)| {
            let receipt = Receipt {
                tx_hash: tx.transaction.hash,
                block_number,
                block_hash,
                tx_index: idx as u32,
                state_root,
                used_gas: U256::from(res.gas_used),
                logs_bloom: Bloom::from(BloomInput::Raw(rlp::encode_list(&res.logs).as_ref())),
                logs: res.logs.clone(),
                log_index,
                code_address: res.code_address,
                sender: tx.sender,
                ret: res.exit_reason.clone(),
                removed: res.removed,
            };
            log_index += res.logs.len() as u32;
            receipt
        })
        .collect::<Vec<_>>();
    let logs = receipts.iter().map(|r| r.logs.clone()).collect::<Vec<_>>();

    (receipts, logs)
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
