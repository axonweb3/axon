use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;

use common_apm::Instant;
use common_apm_derive::trace_span;
use protocol::tokio::{sync::Mutex, time::sleep};
use protocol::traits::{Context, Synchronization, SynchronizationAdapter};
use protocol::types::{Block, Proof, Proposal, Receipt, RichBlock, SignedTransaction, U256};
use protocol::{async_trait, ProtocolResult};

use crate::status::{CurrentStatus, StatusAgent};
use crate::util::digest_signed_transactions;
use crate::ConsensusError;

const POLLING_BROADCAST: u64 = 2000;
const ONCE_SYNC_BLOCK_LIMIT: u64 = 50;

lazy_static::lazy_static! {
    pub static ref SYNC_STATUS: RwLock<SyncStatus> = RwLock::new(SyncStatus::default());
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
    #[trace_span(kind = "consensus.sync", logs = "{remote_number: remote_number}")]
    async fn receive_remote_block(&self, ctx: Context, remote_number: u64) -> ProtocolResult<()> {
        let syncing_lock = self.syncing.try_lock();
        if syncing_lock.is_err() {
            return Ok(());
        }
        if !self.need_sync(remote_number).await? {
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

            self.update_status(ctx, sync_status_agent).await?;
            SYNC_STATUS.write().finish();

            return Err(e);
        }

        log::info!(
            "[synchronization]: sync end, remote block number {:?} current block number {:?}",
            remote_number,
            sync_status.last_number,
        );

        self.update_status(ctx, sync_status_agent).await?;
        SYNC_STATUS.write().finish();

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
            let current_number = self.status.inner().proof.number;
            if current_number != 0 {
                self.adapter
                    .broadcast_number(Context::new(), current_number)
                    .await?;
            }
            sleep(Duration::from_millis(POLLING_BROADCAST)).await;
        }
    }

    #[trace_span(
        kind = "consensus.sync",
        logs = "{
                    current_number: current_number,
                    remote_number: remote_number
                }"
    )]
    async fn start_sync(
        &self,
        ctx: Context,
        sync_status_agent: StatusAgent,
        current_number: u64,
        remote_number: u64,
    ) -> ProtocolResult<()> {
        let mut current_consented_number = current_number;
        let remote_number = if current_number + ONCE_SYNC_BLOCK_LIMIT > remote_number {
            remote_number
        } else {
            current_number + ONCE_SYNC_BLOCK_LIMIT
        };

        {
            SYNC_STATUS
                .write()
                .start(current_consented_number, remote_number);
        }

        while current_consented_number < remote_number {
            let consenting_number = current_consented_number + 1;
            log::info!(
                "[synchronization]: try syncing block, current_consented_number {},syncing_number {}",
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
                consenting_rich_block,
                consenting_proof,
                sync_status_agent.clone(),
            )
            .await
            .map_err(|e| {
                log::error!(
                    "[synchronization]: commit block {} error",
                    current_consented_number
                );
                e
            })?;

            current_consented_number += 1;

            common_apm::metrics::consensus::ENGINE_SYNC_BLOCK_COUNTER.inc_by(1u64);
            common_apm::metrics::consensus::ENGINE_SYNC_BLOCK_HISTOGRAM
                .observe(common_apm::metrics::duration_to_sec(inst.elapsed()));
            SYNC_STATUS.write().add_one();
        }

        Ok(())
    }

    async fn verify_block(
        &self,
        ctx: Context,
        consenting_rich_block: &RichBlock,
    ) -> ProtocolResult<Proof> {
        let consenting_number = consenting_rich_block.block.header.number;
        let proposal = Proposal::new_with_state_root(
            &consenting_rich_block.block.header,
            self.status.inner().last_state_root,
            consenting_rich_block.block.tx_hashes.clone(),
        );

        let consenting_proof: Proof = self
            .adapter
            .get_proof_from_remote(ctx.clone(), consenting_number)
            .await
            .map_err(|e| {
                log::error!(
                    "[synchronization]: get_proof_from_remote error, number {}",
                    consenting_number
                );
                e
            })?;

        self.adapter
            .verify_proof(
                ctx.clone(),
                consenting_rich_block.block.clone(),
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
            .verify_block_header(ctx.clone(), &proposal)
            .await
            .map_err(|e| {
                log::error!(
                    "[synchronization]: verify_block_header error, block header: {:?}",
                    consenting_rich_block.block.header
                );
                e
            })?;

        let previous_block = self
            .adapter
            .get_block_by_number(ctx.clone(), consenting_rich_block.block.header.number - 1)
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
                previous_block.clone(),
                consenting_rich_block.block.header.proof.clone(),
            )
            .await
            .map_err(|e| {
                log::error!(
                    "[synchronization]: verify_proof error, previous block header: {:?}, proof: {:?}",
                    previous_block.header,
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

    #[trace_span(kind = "consensus.sync")]
    async fn commit_block(
        &self,
        ctx: Context,
        rich_block: RichBlock,
        proof: Proof,
        status_agent: StatusAgent,
    ) -> ProtocolResult<()> {
        let block = &rich_block.block;
        let block_hash = block.hash();
        let resp = self
            .adapter
            .exec(
                ctx.clone(),
                status_agent.inner().last_state_root,
                &Proposal::new_with_state_root(
                    &block.header,
                    self.status.inner().last_state_root,
                    block.tx_hashes.clone(),
                ),
                &rich_block.txs,
            )
            .await?;

        if resp.state_root != block.header.state_root {
            return Err(ConsensusError::InvalidStateRoot {
                expect: block.header.state_root,
                actual: resp.state_root,
            }
            .into());
        }

        if resp.receipt_root != block.header.receipts_root {
            return Err(ConsensusError::InvalidReceiptsRoot {
                expect: block.header.receipts_root,
                actual: resp.receipt_root,
            }
            .into());
        }

        let (receipts, _logs) = rich_block.generate_receipts_and_logs(&resp);

        let metadata = self
            .adapter
            .get_metadata_by_block_number(block.header.number)
            .await?;
        let new_status = CurrentStatus {
            prev_hash:       block.hash(),
            last_number:     block.header.number,
            last_state_root: resp.state_root,
            tx_num_limit:    metadata.consensus_config.tx_num_limit,
            max_tx_size:     metadata.consensus_config.max_tx_size.into(),
            proof:           proof.clone(),
        };

        self.save_chain_data(
            ctx.clone(),
            rich_block.txs.clone(),
            receipts,
            rich_block.block.clone(),
        )
        .await?;

        status_agent.swap(new_status);

        // If there are transactions in the transaction pool that have been on chain
        // after this execution, make sure they are cleaned up.
        self.adapter
            .flush_mempool(
                ctx,
                &rich_block.block.tx_hashes,
                rich_block.block.header.number,
            )
            .await?;

        Ok(())
    }

    #[trace_span(kind = "consensus.sync", logs = "{number:number}")]
    async fn get_rich_block_from_remote(
        &self,
        ctx: Context,
        number: u64,
    ) -> ProtocolResult<RichBlock> {
        let block = self.get_block_from_remote(ctx.clone(), number).await?;

        if block.header.number != number {
            log::error!("[synchronization]: block that doesn't match is found");
            return Err(crate::ConsensusError::Other(
                "[synchronization]: block doesn't match request".to_string(),
            )
            .into());
        }

        let mut res = std::collections::BTreeMap::new();
        let mut missing = std::collections::HashMap::new();
        let mut missing_list = Vec::with_capacity(block.tx_hashes.len());
        for (index, hash) in block.tx_hashes.iter().enumerate() {
            if let Some(tx) = self.adapter.get_tx_from_mem(ctx.clone(), hash) {
                res.insert(index, tx);
            } else {
                missing_list.push(*hash);
                missing.insert(hash, index);
            }
        }

        for tx_hashes in missing_list.chunks(self.sync_txs_chunk_size) {
            let remote_txs = self
                .adapter
                .get_txs_from_remote(ctx.clone(), number, tx_hashes)
                .await?;

            for tx in remote_txs {
                if let Some(key) = missing.remove(&tx.transaction.hash) {
                    res.insert(key, tx);
                } else {
                    return Err(crate::ConsensusError::Other(
                        "[synchronization]: Invalid sync tx hash".to_string(),
                    )
                    .into());
                }
            }
        }

        if !missing.is_empty() {
            return Err(crate::ConsensusError::Other(
                "[synchronization]: Unable to get a complete list of transactions ".to_string(),
            )
            .into());
        }

        Ok(RichBlock {
            block,
            txs: res.into_values().collect(),
        })
    }

    #[trace_span(kind = "consensus.sync", logs = "{number:number}")]
    async fn get_block_from_remote(&self, ctx: Context, number: u64) -> ProtocolResult<Block> {
        self.adapter
            .get_block_from_remote(ctx.clone(), number)
            .await
    }

    #[trace_span(
        kind = "consensus.sync",
        logs = "{txs_len:
    txs.len()}"
    )]
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

    async fn need_sync(&self, remote_number: u64) -> ProtocolResult<bool> {
        let mut current_number = self.status.inner().last_number;
        if remote_number == 0 {
            return Ok(false);
        }

        if remote_number <= current_number {
            return Ok(false);
        }

        if current_number == remote_number - 1 {
            sleep(Duration::from_millis(
                self.adapter
                    .get_metadata_by_block_number(current_number)
                    .await?
                    .consensus_config
                    .interval,
            ))
            .await;

            current_number = self.status.inner().last_number;
            if current_number == remote_number {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn update_status(
        &self,
        ctx: Context,
        sync_status_agent: StatusAgent,
    ) -> ProtocolResult<()> {
        let sync_status = sync_status_agent.inner();
        self.status.swap(sync_status.clone());
        let metadata = self
            .adapter
            .get_metadata_by_block_number(sync_status.last_number + 1)
            .await?;

        self.adapter.update_status(
            ctx,
            sync_status.last_number,
            metadata.consensus_config.interval,
            metadata.consensus_config.propose_ratio,
            metadata.consensus_config.prevote_ratio,
            metadata.consensus_config.precommit_ratio,
            metadata.consensus_config.brake_ratio,
            metadata.verifier_list.into_iter().map(Into::into).collect(),
        )?;

        log::info!(
            "[synchronization]: synced block, status: number:{}",
            sync_status.last_number,
        );
        Ok(())
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub enum SyncStatus {
    #[default]
    False,
    Syncing {
        start:   U256,
        current: U256,
        highest: U256,
    },
}

impl SyncStatus {
    pub fn start(&mut self, start: u64, highest: u64) {
        *self = SyncStatus::Syncing {
            start:   start.into(),
            current: start.into(),
            highest: highest.into(),
        };
    }

    pub fn finish(&mut self) {
        *self = SyncStatus::False;
    }

    pub fn add_one(&mut self) {
        match *self {
            SyncStatus::False => (),
            SyncStatus::Syncing {
                ref mut current, ..
            } => {
                *current += U256::one();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::synchronization::{get_mock_rick_block, get_mock_synchronization};
    use protocol::tokio;

    use super::*;

    #[tokio::test]
    async fn test_start_sync() {
        let synchronization = get_mock_synchronization();
        let ctx = Context::default();
        let status_agent = StatusAgent::new(CurrentStatus::default());
        let result = synchronization.start_sync(ctx, status_agent, 0, 5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_block() {
        let synchronization = get_mock_synchronization();
        let ctx = Context::default();
        let genesis = get_mock_rick_block();
        let result = synchronization.verify_block(ctx, &genesis).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_commit_block() {
        let sync = get_mock_synchronization();
        let ctx = Context::default();
        let rich_block = get_mock_rick_block();
        let proof = Proof::default();
        let status_agent = StatusAgent::new(CurrentStatus::default());
        let result = sync
            .commit_block(ctx, rich_block, proof, status_agent)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_rich_block_from_remote() {
        let sync = get_mock_synchronization();
        let ctx = Context::default();
        let result = sync.get_rich_block_from_remote(ctx, 4).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_need_sync() {
        let sync = get_mock_synchronization();

        let result = sync.need_sync(4).await.unwrap();
        assert!(result);

        let result = sync.need_sync(0).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_update_status() {
        let sync = get_mock_synchronization();
        let ctx = Context::default();

        let sync_status_agent = StatusAgent::new(CurrentStatus::default());
        let result = sync.update_status(ctx, sync_status_agent).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_sync_status() {
        let mut sync_status = SyncStatus::default();
        assert_eq!(sync_status, SyncStatus::False);

        sync_status.add_one();
        assert_eq!(sync_status, SyncStatus::False);

        sync_status.start(0, 5);
        assert_eq!(sync_status, SyncStatus::Syncing {
            start:   U256::zero(),
            current: U256::zero(),
            highest: U256::from(5),
        });

        sync_status.add_one();
        assert_eq!(sync_status, SyncStatus::Syncing {
            start:   U256::zero(),
            current: U256::from(1),
            highest: U256::from(5),
        });

        sync_status.finish();
        assert_eq!(sync_status, SyncStatus::False);
    }
}
