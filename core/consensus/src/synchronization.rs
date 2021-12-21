use std::sync::Arc;
use std::time::{Duration, Instant};

// use common_apm::muta_apm;
use protocol::codec::ProtocolCodec;
use protocol::tokio::{sync::Mutex, time::sleep};
use protocol::traits::{Context, Synchronization, SynchronizationAdapter};
use protocol::types::{Block, Bloom, BloomInput, Hasher, Proof, Receipt, SignedTransaction};
use protocol::{async_trait, ProtocolResult};

use crate::status::{CurrentStatus, StatusAgent, METADATA_CONTROLER};
use crate::util::digest_signed_transactions;
use crate::{engine::generate_receipts_and_logs, ConsensusError};

const POLLING_BROADCAST: u64 = 2000;
const ONCE_SYNC_BLOCK_LIMIT: u64 = 50;

#[derive(Clone, Debug)]
pub struct RichBlock {
    pub block: Block,
    pub txs:   Vec<SignedTransaction>,
}

pub struct OverlordSynchronization<Adapter: SynchronizationAdapter> {
    adapter: Arc<Adapter>,
    status:  StatusAgent,
    lock:    Arc<Mutex<()>>,
    syncing: Mutex<()>,

    sync_txs_chunk_size: usize,
}

#[async_trait]
impl<Adapter: SynchronizationAdapter> Synchronization for OverlordSynchronization<Adapter> {
    // #[muta_apm::derive::tracing_span(
    //     kind = "consensus.sync",
    //     logs = "{'remote_number': 'remote_number'}"
    // )]
    async fn receive_remote_block(&self, ctx: Context, remote_number: u64) -> ProtocolResult<()> {
        let syncing_lock = self.syncing.try_lock();
        if syncing_lock.is_err() {
            return Ok(());
        }
        if !self.need_sync(ctx.clone(), remote_number).await? {
            return Ok(());
        }

        // Lock the consensus engine, block commit process.
        let commit_lock = self.lock.try_lock();
        if commit_lock.is_err() {
            return Ok(());
        }

        let current_number = self.status.inner().last_number;

        if remote_number <= current_number {
            return Ok(());
        }

        let remote_number = remote_number - 1;
        log::info!(
            "[synchronization]: sync start, remote block number {:?} current block number {:?}",
            remote_number,
            current_number,
        );

        let sync_status_agent = self.init_status_agent().await?;
        let sync_resp = self
            .start_sync(
                ctx.clone(),
                sync_status_agent.clone(),
                current_number,
                remote_number,
            )
            .await;
        let sync_status = sync_status_agent.inner();

        if let Err(e) = sync_resp {
            log::error!(
                "[synchronization]: err, current_number {:?} err_msg: {:?}",
                sync_status.last_number,
                e
            );
            return Err(e);
        }

        log::info!(
            "[synchronization]: sync end, remote block number {:?} current block number {:?}",
            remote_number,
            sync_status.last_number,
        );

        self.update_status(ctx, sync_status_agent)?;

        Ok(())
    }
}

impl<Adapter: SynchronizationAdapter> OverlordSynchronization<Adapter> {
    pub fn new(
        sync_txs_chunk_size: usize,
        adapter: Arc<Adapter>,
        status: StatusAgent,
        lock: Arc<Mutex<()>>,
    ) -> Self {
        let syncing = Mutex::new(());

        Self {
            adapter,
            status,
            lock,
            syncing,

            sync_txs_chunk_size,
        }
    }

    pub async fn polling_broadcast(&self) -> ProtocolResult<()> {
        loop {
            let current_number = self.status.inner().last_number;
            if current_number != 0 {
                self.adapter
                    .broadcast_number(Context::new(), current_number)
                    .await?;
            }
            sleep(Duration::from_millis(POLLING_BROADCAST)).await;
        }
    }

    // #[muta_apm::derive::tracing_span(
    //     kind = "consensus.sync",
    //     logs = "{'current_number': 'current_number', 'remote_number':
    // 'remote_number'}" )]
    async fn start_sync(
        &self,
        ctx: Context,
        sync_status_agent: StatusAgent,
        current_number: u64,
        remote_number: u64,
    ) -> ProtocolResult<()> {
        let remote_number = if current_number + ONCE_SYNC_BLOCK_LIMIT > remote_number {
            remote_number
        } else {
            current_number + ONCE_SYNC_BLOCK_LIMIT
        };

        let mut current_consented_number = current_number;

        while current_consented_number < remote_number {
            let consenting_number = current_consented_number + 1;
            log::info!(
                "[synchronization]: try syncing block, current_consented_number:{},syncing_number:{}",
                current_consented_number,
                consenting_number
            );

            let consenting_rich_block: RichBlock = self
                .get_rich_block_from_remote(ctx.clone(), consenting_number)
                .await
                .map_err(|e| {
                    log::error!(
                        "[synchronization]: get_rich_block_from_remote error, number: {:?}",
                        consenting_number
                    );
                    e
                })?;

            let consenting_proof = self
                .verify_block(ctx.clone(), &consenting_rich_block)
                .await?;

            let inst = Instant::now();
            self.commit_block(
                ctx.clone(),
                consenting_rich_block.clone(),
                consenting_proof,
                sync_status_agent.clone(),
            )
            .await
            .map_err(|e| {
                log::error!(
                    "[synchronization]: commit block {} error",
                    consenting_rich_block.block.header.number
                );
                e
            })?;

            current_consented_number += 1;

            common_apm::metrics::consensus::ENGINE_SYNC_BLOCK_COUNTER.inc_by(1u64);
            common_apm::metrics::consensus::ENGINE_SYNC_BLOCK_HISTOGRAM
                .observe(common_apm::metrics::duration_to_sec(inst.elapsed()));
        }

        Ok(())
    }

    async fn verify_block(
        &self,
        ctx: Context,
        consenting_rich_block: &RichBlock,
    ) -> ProtocolResult<Proof> {
        let consenting_number = consenting_rich_block.block.header.number;

        let consenting_proof: Proof = self
            .adapter
            .get_proof_from_remote(ctx.clone(), consenting_number)
            .await
            .map_err(|e| {
                log::error!(
                    "[synchronization]: get_proof_from_remote error, number: {:?}",
                    consenting_number
                );
                e
            })?;

        self.adapter
            .verify_proof(
                ctx.clone(),
                &consenting_rich_block.block.header,
                consenting_proof.clone(),
            )
            .await
            .map_err(|e| {
                log::error!(
                    "[synchronization]: verify_proof error, syncing block header: {:?}, proof: {:?}",
                    consenting_rich_block.block.header,
                    consenting_proof,
                );
                e
            })?;

        self.adapter
            .verify_block_header(ctx.clone(), &consenting_rich_block.block)
            .await
            .map_err(|e| {
                log::error!(
                    "[synchronization]: verify_block_header error, block header: {:?}",
                    consenting_rich_block.block.header
                );
                e
            })?;

        let previous_block_header = self
            .adapter
            .get_block_header_by_number(ctx.clone(), consenting_rich_block.block.header.number - 1)
            .await
            .map_err(|e| {
                log::error!(
                    "[synchronization] get previous block {} error",
                    consenting_rich_block.block.header.number - 1
                );
                e
            })?;

        self.adapter
            .verify_proof(
                ctx.clone(),
                &previous_block_header,
                consenting_rich_block.block.header.proof.clone(),
            )
            .await
            .map_err(|e| {
                log::error!(
                    "[synchronization]: verify_proof error, previous block header: {:?}, proof: {:?}",
                    previous_block_header,
                    consenting_rich_block.block.header.proof
                );
                e
            })?;

        let signed_txs_hash = digest_signed_transactions(&consenting_rich_block.txs);
        if signed_txs_hash != consenting_rich_block.block.header.signed_txs_hash {
            return Err(ConsensusError::InvalidOrderSignedTransactionsHash {
                expect: signed_txs_hash,
                actual: consenting_rich_block.block.header.signed_txs_hash,
            }
            .into());
        }

        Ok(consenting_proof)
    }

    async fn init_status_agent(&self) -> ProtocolResult<StatusAgent> {
        Ok(StatusAgent::new(self.status.inner()))
    }

    // #[muta_apm::derive::tracing_span(kind = "consensus.sync")]
    async fn commit_block(
        &self,
        ctx: Context,
        rich_block: RichBlock,
        proof: Proof,
        status_agent: StatusAgent,
    ) -> ProtocolResult<()> {
        let block = &rich_block.block;
        let block_hash = Hasher::digest(block.header.encode()?);
        let resp = self
            .adapter
            .exec(
                ctx.clone(),
                block_hash,
                &block.header,
                rich_block.txs.clone(),
            )
            .await?;

        let (receipts, logs) = generate_receipts_and_logs(
            block.header.number,
            block_hash,
            block.header.state_root,
            &rich_block.txs,
            &resp,
        );
        let metadata = METADATA_CONTROLER.load().current();

        let new_status = CurrentStatus {
            prev_hash:        Hasher::digest(block.header.encode()?),
            last_number:      block.header.number,
            state_root:       resp.state_root,
            receipts_root:    resp.receipt_root,
            log_bloom:        Bloom::from(BloomInput::Raw(rlp::encode_list(&logs).as_ref())),
            gas_limit:        metadata.gas_limit.into(),
            gas_used:         resp.gas_used.into(),
            base_fee_per_gas: None,
            proof:            proof.clone(),
        };

        status_agent.swap(new_status);

        self.save_chain_data(
            ctx.clone(),
            rich_block.txs.clone(),
            receipts,
            rich_block.block.clone(),
        )
        .await?;

        // If there are transactions in the trasnaction pool that have been on chain
        // after this execution, make sure they are cleaned up.
        self.adapter
            .flush_mempool(ctx.clone(), &rich_block.block.tx_hashes)
            .await?;

        Ok(())
    }

    // #[muta_apm::derive::tracing_span(kind = "consensus.sync", logs = "{'number':
    // 'number'}")]
    async fn get_rich_block_from_remote(
        &self,
        ctx: Context,
        number: u64,
    ) -> ProtocolResult<RichBlock> {
        let block = self.get_block_from_remote(ctx.clone(), number).await?;

        let mut txs = Vec::with_capacity(block.tx_hashes.len());

        for tx_hashes in block.tx_hashes.chunks(self.sync_txs_chunk_size) {
            let remote_txs = self
                .adapter
                .get_txs_from_remote(ctx.clone(), number, tx_hashes)
                .await?;

            txs.extend(remote_txs);
        }

        Ok(RichBlock { block, txs })
    }

    // #[muta_apm::derive::tracing_span(kind = "consensus.sync", logs = "{'number':
    // 'number'}")]
    async fn get_block_from_remote(&self, ctx: Context, number: u64) -> ProtocolResult<Block> {
        self.adapter
            .get_block_from_remote(ctx.clone(), number)
            .await
    }

    // #[muta_apm::derive::tracing_span(kind = "consensus.sync", logs = "{'txs_len':
    // 'txs.len()'}")]
    async fn save_chain_data(
        &self,
        ctx: Context,
        txs: Vec<SignedTransaction>,
        receipts: Vec<Receipt>,
        block: Block,
    ) -> ProtocolResult<()> {
        self.adapter
            .save_signed_txs(ctx.clone(), block.header.number, txs)
            .await?;
        self.adapter
            .save_receipts(ctx.clone(), block.header.number, receipts)
            .await?;
        self.adapter
            .save_proof(ctx.clone(), block.header.proof.clone())
            .await?;
        self.adapter.save_block(ctx.clone(), block).await?;
        Ok(())
    }

    async fn need_sync(&self, ctx: Context, remote_number: u64) -> ProtocolResult<bool> {
        let mut current_number = self.status.inner().last_number;
        if remote_number == 0 {
            return Ok(false);
        }

        if remote_number <= current_number {
            return Ok(false);
        }

        if current_number == remote_number - 1 {
            sleep(Duration::from_millis(
                METADATA_CONTROLER.load().current().interval,
            ))
            .await;

            current_number = self.status.inner().last_number;
            if current_number == remote_number {
                return Ok(false);
            }
        }

        let block = self
            .get_block_from_remote(ctx.clone(), remote_number)
            .await?;

        log::debug!(
            "[synchronization] get block from remote success {:?} ",
            remote_number
        );

        if block.header.number != remote_number {
            log::error!("[synchronization]: block that doesn't match is found");
            return Ok(false);
        }

        Ok(true)
    }

    fn update_status(&self, ctx: Context, sync_status_agent: StatusAgent) -> ProtocolResult<()> {
        let sync_status = sync_status_agent.inner();
        self.status.swap(sync_status.clone());
        let metadata = METADATA_CONTROLER.load().current();

        self.adapter.update_status(
            ctx,
            sync_status.last_number,
            metadata.interval,
            metadata.propose_ratio,
            metadata.prevote_ratio,
            metadata.precommit_ratio,
            metadata.brake_ratio,
            metadata.verifier_list.into_iter().map(Into::into).collect(),
        )?;

        log::info!(
            "[synchronization]: synced block, status: number:{}",
            sync_status.last_number,
        );
        Ok(())
    }
}
