use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};

use json::JsonValue;
use log::{error, info, warn};
use overlord::error::ConsensusError as OverlordError;
use overlord::types::{Commit, Node, OverlordMsg, Status, ViewChangeReason};
use overlord::{Consensus as Engine, Wal};
use parking_lot::RwLock;
use rlp::Encodable;

// use common_apm::muta_apm;
use common_crypto::BlsPublicKey;
use common_logger::{json, log};
use common_merkle::Merkle;

use protocol::codec::ProtocolCodec;
use protocol::traits::{ConsensusAdapter, Context, MessageTarget, NodeInfo};
use protocol::types::{
    Block, BlockNumber, Bloom, BloomInput, Bytes, ExecResp, Hash, Hasher, Header, MerkleRoot,
    Metadata, Pill, Proof, Receipt, SignedTransaction, ValidatorExtend, U256,
};
use protocol::{
    async_trait, tokio, tokio::sync::Mutex as AsyncMutex, tokio::time::sleep, ProtocolError,
    ProtocolResult,
};

use crate::message::{
    END_GOSSIP_AGGREGATED_VOTE, END_GOSSIP_SIGNED_CHOKE, END_GOSSIP_SIGNED_PROPOSAL,
    END_GOSSIP_SIGNED_VOTE,
};
use crate::status::{CurrentStatus, StatusAgent};
use crate::util::{digest_signed_transactions, time_now, OverlordCrypto};
use crate::wal::{ConsensusWal, SignedTxsWAL};
use crate::{ConsensusError, METADATA_CONTROLER};

const RETRY_CHECK_ROOT_LIMIT: u8 = 15;
const RETRY_CHECK_ROOT_INTERVAL: u64 = 100; // 100ms

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
}

#[async_trait]
impl<Adapter: ConsensusAdapter + 'static> Engine<Pill> for ConsensusEngine<Adapter> {
    // #[muta_apm::derive::tracing_span(
    //     kind = "consensus.engine",
    //     logs = "{'next_number': 'next_number'}"
    // )]
    async fn get_block(
        &self,
        ctx: Context,
        next_number: u64,
    ) -> Result<(Pill, Bytes), Box<dyn Error + Send>> {
        let status = self.status.inner();
        let (tx_hashes, propose_hashes) = self
            .adapter
            .get_txs_from_mempool(ctx.clone(), next_number, status.gas_limit.as_u64(), 10000)
            .await?
            .clap();
        let signed_txs = self.adapter.get_full_txs(ctx.clone(), &tx_hashes).await?;
        let order_root = Merkle::from_hashes(tx_hashes.clone())
            .get_root_hash()
            .unwrap_or_default();

        let header = Header {
            prev_hash:         status.prev_hash,
            proposer:          self.node_info.self_address.0,
            state_root:        status.state_root,
            transactions_root: order_root,
            signed_txs_hash:   digest_signed_transactions(&signed_txs),
            receipts_root:     status.receipts_root,
            log_bloom:         status.log_bloom,
            difficulty:        Default::default(),
            timestamp:         time_now(),
            number:            next_number,
            gas_used:          status.gas_used,
            gas_limit:         status.gas_limit,
            extra_data:        Default::default(),
            mixed_hash:        None,
            nonce:             Default::default(),
            base_fee_per_gas:  status.base_fee_per_gas,
            proof:             status.proof,
            chain_id:          self.node_info.chain_id,
        };

        if header.number != header.proof.number + 1 {
            error!(
                "[consensus] get_block for {}, proof error, proof number {} mismatch",
                header.number, header.proof.number,
            );
        }

        let block = Block { header, tx_hashes };

        let pill = Pill {
            block,
            propose_hashes,
        };

        let hash = Hasher::digest(pill.block.header.encode()?);
        let mut set = self.exemption_hash.write();
        set.insert(hash);

        Ok((pill, Bytes::from(hash.as_bytes().to_vec())))
    }

    // #[muta_apm::derive::tracing_span(
    //     kind = "consensus.engine",
    //     logs = "{'next_number': 'next_number', 'hash':
    // 'Hash::from_bytes(hash.clone()).unwrap().as_hex()', 'txs_len':
    // 'block.inner.block.ordered_tx_hashes.len()'}"
    // )]
    async fn check_block(
        &self,
        ctx: Context,
        next_number: u64,
        hash: Bytes,
        block: Pill,
    ) -> Result<(), Box<dyn Error + Send>> {
        let time = Instant::now();

        if block.block.header.number != block.block.header.proof.number + 1 {
            error!("[consensus-engine]: check_block for overlord receives a proposal, error, block number {}, block {:?}", block.block.header.number,block.block);
        }

        let tx_hashes = block.block.tx_hashes.clone();
        let tx_hashes_len = tx_hashes.len();
        let exemption = {
            self.exemption_hash
                .read()
                .contains(&Hash::from_slice(hash.as_ref()))
        };
        let sync_tx_hashes = block.propose_hashes.clone();
        let pill = block;

        gauge_txs_len(&pill);

        // If the block is proposed by self, it does not need to check. Get full signed
        // transactions directly.
        if !exemption {
            if let Err(e) = self.inner_check_block(ctx.clone(), &pill.block).await {
                let mut reason = self.last_check_block_fail_reason.write();
                *reason = e.to_string();
                return Err(e.into());
            }

            let adapter = Arc::clone(&self.adapter);
            let ctx_clone = ctx.clone();
            tokio::spawn(async move {
                if let Err(e) = sync_txs(ctx_clone, adapter, sync_tx_hashes).await {
                    error!("Consensus sync block error {}", e);
                }
            });
        }

        info!(
            "[consensus-engine]: check block cost {:?}",
            Instant::now() - time
        );
        let time = Instant::now();
        let txs = self.adapter.get_full_txs(ctx, &tx_hashes).await?;

        info!(
            "[consensus-engine]: get txs cost {:?}",
            Instant::now() - time
        );
        let time = Instant::now();
        self.txs_wal
            .save(next_number, pill.block.header.transactions_root, txs)?;

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
    // #[muta_apm::derive::tracing_span(
    //     kind = "consensus.engine",
    //     logs = "{'current_number': 'current_number', 'txs_len':
    // 'commit.content.inner.block.ordered_tx_hashes.len()'}"
    // )]
    async fn commit(
        &self,
        ctx: Context,
        current_number: u64,
        commit: Commit<Pill>,
    ) -> Result<Status, Box<dyn Error + Send>> {
        let lock = self.lock.try_lock();
        if lock.is_err() {
            return Err(ProtocolError::from(ConsensusError::LockInSync).into());
        }

        let status = self.status.inner();
        let metadata = METADATA_CONTROLER.get().unwrap().current();

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

        let pill = commit.content;
        let block = pill.block;
        let block_hash = Hash::from_slice(commit.proof.block_hash.as_ref());
        let signature = commit.proof.signature.signature.clone();
        let bitmap = commit.proof.signature.address_bitmap.clone();
        let txs_len = block.tx_hashes.len();

        // Storage save the latest proof.
        let proof = Proof {
            number: commit.proof.height,
            round: commit.proof.round,
            block_hash,
            signature,
            bitmap,
        };
        common_apm::metrics::consensus::ENGINE_ROUND_GAUGE.set(commit.proof.round as i64);

        self.adapter.save_proof(ctx.clone(), proof.clone()).await?;

        // Get full transactions from mempool. If is error, try get from wal.
        let signed_txs = match self
            .adapter
            .get_full_txs(ctx.clone(), &block.tx_hashes)
            .await
        {
            Ok(txs) => txs,
            Err(_) => self
                .txs_wal
                .load(current_number, block.header.transactions_root)?,
        };

        // Execute transactions
        let resp = self
            .adapter
            .exec(
                ctx.clone(),
                Hasher::digest(block.header.encode()?),
                &block.header,
                signed_txs.clone(),
            )
            .await?;

        info!(
            "[consensus]: validator of number {} is {:?}",
            current_number + 1,
            metadata.verifier_list
        );

        self.update_status(resp, block.clone(), proof, signed_txs)
            .await?;

        self.adapter
            .flush_mempool(ctx.clone(), &block.tx_hashes)
            .await?;

        self.txs_wal.remove(current_number.saturating_sub(2))?;

        {
            self.exemption_hash.write().clear();
        }

        self.update_metadata(current_number + 1);
        let metadata = METADATA_CONTROLER.get().unwrap().current();

        let status = Status {
            height:         current_number + 1,
            interval:       Some(metadata.interval),
            authority_list: convert_to_overlord_authority(&metadata.verifier_list),
            timer_config:   Some(metadata.into()),
        };

        self.adapter
            .broadcast_number(ctx.clone(), current_number)
            .await?;

        self.metric_commit(current_number, txs_len);

        Ok(status)
    }

    /// Only signed proposal and aggregated vote will be broadcast to others.
    // #[muta_apm::derive::tracing_span(kind = "consensus.engine")]
    async fn broadcast_to_other(
        &self,
        ctx: Context,
        msg: OverlordMsg<Pill>,
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
    // #[muta_apm::derive::tracing_span(
    //     kind = "consensus.engine",
    //     logs = "{'pub_key': 'hex::encode(pub_key.clone())'}"
    // )]
    async fn transmit_to_relayer(
        &self,
        ctx: Context,
        pub_key: Bytes,
        msg: OverlordMsg<Pill>,
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
    // #[muta_apm::derive::tracing_span(
    //     kind = "consensus.engine",
    //     logs = "{'next_number': 'next_number'}"
    // )]
    async fn get_authority_list(
        &self,
        _ctx: Context,
        next_number: u64,
    ) -> Result<Vec<Node>, Box<dyn Error + Send>> {
        if next_number == 0 {
            return Ok(vec![]);
        }

        let current_metadata = METADATA_CONTROLER.get().unwrap().current();
        let old_metadata = if current_metadata.version.contains(next_number - 1) {
            current_metadata
        } else {
            METADATA_CONTROLER.get().unwrap().previous()
        };

        let mut old_validators = old_metadata
            .verifier_list
            .into_iter()
            .map(|v| Node {
                address:        v.pub_key.decode(),
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
        }
    }

    pub fn status(&self) -> CurrentStatus {
        self.status.inner()
    }

    fn update_metadata(&self, block_number: BlockNumber) {
        METADATA_CONTROLER.get().unwrap().update(block_number);
    }

    async fn inner_check_block(&self, ctx: Context, block: &Block) -> ProtocolResult<()> {
        let current_timestamp = time_now();

        self.adapter
            .verify_block_header(ctx.clone(), block)
            .await
            .map_err(|e| {
                error!(
                    "[consensus] check_block, verify_block_header error, block header: {:?}",
                    block.header
                );
                e
            })?;

        // verify the proof in the block for previous block
        // skip to get previous proof to compare because the node may just comes from
        // sync and waste a delay of read
        let previous_block_header = self
            .adapter
            .get_block_header_by_number(ctx.clone(), block.header.number - 1)
            .await?;

        // verify block timestamp.
        if !validate_timestamp(
            current_timestamp,
            block.header.timestamp,
            previous_block_header.timestamp,
        ) {
            return Err(ProtocolError::from(ConsensusError::InvalidTimestamp));
        }

        self.adapter
                .verify_proof(
                    ctx.clone(),
                    &previous_block_header,
                    block.header.proof.clone(),
                )
                .await
                .map_err(|e| {
                    error!(
                        "[consensus] check_block, verify_proof error, previous block header: {:?}, proof: {:?}",
                        previous_block_header,
                        block.header.proof
                    );
                    e
                })?;

        self.adapter
            .verify_txs(ctx.clone(), block.header.number, &block.tx_hashes)
            .await
            .map_err(|e| {
                error!("[consensus] check_block, verify_txs error",);
                e
            })?;

        // If it is inconsistent with the state of the proposal, we will wait for a
        // period of time.
        let mut check_retry = 0;
        loop {
            match self.check_block_roots(ctx.clone(), &block.header) {
                Ok(()) => break,
                Err(e) => {
                    if check_retry >= RETRY_CHECK_ROOT_LIMIT {
                        return Err(e);
                    }

                    check_retry += 1;
                }
            }
            sleep(Duration::from_millis(RETRY_CHECK_ROOT_INTERVAL)).await;
        }

        let signed_txs = self
            .adapter
            .get_full_txs(ctx.clone(), &block.tx_hashes)
            .await?;
        self.check_order_transactions(ctx.clone(), block, &signed_txs)
    }

    // #[muta_apm::derive::tracing_span(kind = "consensus.engine")]
    fn check_block_roots(&self, _ctx: Context, block: &Header) -> ProtocolResult<()> {
        let status = self.status.inner();
        if status.prev_hash != block.prev_hash {
            return Err(ConsensusError::InvalidPrevhash {
                expect: status.prev_hash,
                actual: block.prev_hash,
            }
            .into());
        }

        // check state root
        if status.state_root != block.state_root {
            warn!(
                "invalid status list_state_root, latest {:?}, block {:?}",
                status.state_root, block.state_root
            );
            return Err(ConsensusError::InvalidStatusVec.into());
        }

        // check receipt root
        if status.receipts_root != block.receipts_root {
            error!(
                "current list receipt root {:?}, block receipt root {:?}",
                status.receipts_root, block.receipts_root
            );
            return Err(ConsensusError::InvalidStatusVec.into());
        }

        // check cycles used
        if status.gas_used != block.gas_used {
            error!(
                "current list cycles used {:?}, block cycles used {:?}",
                status.gas_used, block.gas_used
            );
            return Err(ConsensusError::InvalidStatusVec.into());
        }

        Ok(())
    }

    // #[muta_apm::derive::tracing_span(
    //     kind = "consensus.engine",
    //     logs = "{'txs_len': 'signed_txs.len()'}"
    // )]
    fn check_order_transactions(
        &self,
        _ctx: Context,
        block: &Block,
        signed_txs: &[SignedTransaction],
    ) -> ProtocolResult<()> {
        let order_root = Merkle::from_hashes(block.tx_hashes.clone())
            .get_root_hash()
            .unwrap_or_default();

        let stxs_hash = Hasher::digest(rlp::encode_list(signed_txs));

        if stxs_hash != block.header.signed_txs_hash {
            return Err(ConsensusError::InvalidOrderSignedTransactionsHash {
                expect: stxs_hash,
                actual: block.header.signed_txs_hash,
            }
            .into());
        }

        if order_root != block.header.transactions_root {
            return Err(ConsensusError::InvalidOrderRoot {
                expect: order_root,
                actual: block.header.transactions_root,
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
        resp: ExecResp,
        block: Block,
        proof: Proof,
        txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<()> {
        let block_number = block.header.number;

        let (receipts, logs) = generate_receipts_and_logs(block.header.state_root, &txs, &resp);

        // Save signed transactions
        self.adapter
            .save_signed_txs(Context::new(), block_number, txs)
            .await?;

        // Save the block.
        self.adapter
            .save_block(Context::new(), block.clone())
            .await?;

        self.adapter
            .save_receipts(Context::new(), block_number, receipts)
            .await?;

        self.adapter
            .save_proof(Context::new(), block.header.proof.clone())
            .await?;

        let metadata = METADATA_CONTROLER.get().unwrap().current();
        let new_status = CurrentStatus {
            prev_hash:        Hasher::digest(block.header.encode()?),
            last_number:      block_number,
            state_root:       resp.state_root,
            receipts_root:    resp.receipt_root,
            log_bloom:        Bloom::from(BloomInput::Raw(rlp::encode_list(&logs).as_ref())),
            gas_limit:        metadata.gas_limit.into(),
            gas_used:         resp.gas_used.into(),
            base_fee_per_gas: None,
            proof:            proof.clone(),
        };

        self.status.swap(new_status);

        // update timeout_gap of mempool
        self.adapter.set_args(
            Context::new(),
            metadata.timeout_gap,
            metadata.gas_limit,
            metadata.max_tx_size,
        );

        let pub_keys = metadata
            .verifier_list
            .iter()
            .map(|v| v.pub_key.decode())
            .collect();
        self.adapter.tag_consensus(Context::new(), pub_keys)?;

        if block.header.number != proof.number {
            info!("[consensus] update_status for handle_commit, error, before update, block number {}, proof number:{}, proof : {:?}",
            block.header.number,
            proof.number,
            proof.clone());
        }

        self.update_overlord_crypto(metadata)?;
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
        let elapsed = (now - last_commit_time) as f64;
        common_apm::metrics::consensus::ENGINE_CONSENSUS_COST_TIME.observe(elapsed / 1e3);
        let mut last_commit_time = self.last_commit_time.write();
        *last_commit_time = now;
    }
}

pub fn generate_new_crypto_map(metadata: Metadata) -> ProtocolResult<HashMap<Bytes, BlsPublicKey>> {
    let mut new_addr_pubkey_map = HashMap::new();
    for validator in metadata.verifier_list.into_iter() {
        let addr = validator.pub_key.decode();
        let hex_pubkey = hex::decode(validator.bls_pub_key.as_string_trim0x()).map_err(|err| {
            ConsensusError::Other(format!("hex decode metadata bls pubkey error {:?}", err))
        })?;
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
            address:        v.pub_key.decode(),
            propose_weight: v.propose_weight,
            vote_weight:    v.vote_weight,
        })
        .collect::<Vec<_>>();
    authority.sort();
    authority
}

async fn sync_txs<CA: ConsensusAdapter>(
    ctx: Context,
    adapter: Arc<CA>,
    propose_hashes: Vec<Hash>,
) -> ProtocolResult<()> {
    adapter.sync_txs(ctx, propose_hashes).await
}

fn validate_timestamp(
    current_timestamp: u64,
    proposal_timestamp: u64,
    previous_timestamp: u64,
) -> bool {
    if proposal_timestamp < previous_timestamp {
        return false;
    }

    if proposal_timestamp > current_timestamp {
        return false;
    }

    true
}

pub fn generate_receipts_and_logs(
    state_root: MerkleRoot,
    txs: &[SignedTransaction],
    resp: &ExecResp,
) -> (Vec<Receipt>, Vec<Bloom>) {
    let receipts = txs
        .iter()
        .zip(resp.tx_resp.iter())
        .map(|(tx, res)| Receipt {
            tx_hash: tx.transaction.hash,
            state_root,
            used_gas: U256::from(res.gas_used),
            logs_bloom: Bloom::from(BloomInput::Raw(rlp::encode_list(&res.logs).as_ref())),
            logs: res.logs.clone(),
        })
        .collect::<Vec<_>>();
    let logs = receipts.iter().map(|r| r.logs_bloom).collect::<Vec<_>>();

    (receipts, logs)
}

fn gauge_txs_len(pill: &Pill) {
    common_apm::metrics::consensus::ENGINE_ORDER_TX_GAUGE.set(pill.block.tx_hashes.len() as i64);
    common_apm::metrics::consensus::ENGINE_SYNC_TX_GAUGE.set(pill.propose_hashes.len() as i64);
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
