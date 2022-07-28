use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::{
    status::{CurrentStatus, StatusAgent},
    synchronization::{OverlordSynchronization, RichBlock},
    // util::time_now,
    // OverlordConsensusAdapter,
};
// use core_network::{NetworkConfig, NetworkService};
use creep::Context;
use protocol::{
    async_trait,
    codec::{hex_decode, hex_encode},
    tokio::{self, sync::Mutex as AsyncMutex},
    traits::{CommonConsensusAdapter, Synchronization, SynchronizationAdapter},
    types::{
        Block, BlockNumber, Bytes, Eip1559Transaction, ExecResp, Header, Hex, Log, MerkleRoot,
        Metadata, Proof, Proposal, Public, Receipt, SignatureComponents, SignedTransaction,
        TransactionAction, UnsignedTransaction, UnverifiedTransaction, Validator, H160, H256, U256,
    },
    ProtocolResult,
};
use protocol::{types::Hash, ProtocolError};

// fn mock_network_service() -> NetworkService {

//     let peer_id = "QmNk6bBwkLPuqnsrtxpp819XLZY3ymgjs3p1nKtxBVgqxj";
//     let peer_conf = NetworkConfig::new()
//         .listen_addr("/ip4/127.0.0.1/tcp/1338".parse().unwrap())
//         .bootstraps(vec![format!("/ip4/127.0.0.1/tcp/1337/p2p/{}", peer_id)
//             .parse()
//             .unwrap()]);
//     NetworkService::new(peer_conf)
// }

#[derive(Debug, Default)]
pub struct MockSyncAdapter {}

fn gen_tx(sender: H160, addr: H160, value: u64, data: Vec<u8>) -> SignedTransaction {
    SignedTransaction {
        transaction: UnverifiedTransaction {
            unsigned:  UnsignedTransaction::Eip1559(Eip1559Transaction {
                nonce:                    U256::default(),
                max_priority_fee_per_gas: U256::default(),
                gas_price:                U256::default(),
                gas_limit:                U256::from_str("0x1000000000").unwrap(),
                action:                   TransactionAction::Call(addr),
                value:                    value.into(),
                data:                     data.into(),
                access_list:              Vec::new(),
            }),
            signature: Some(SignatureComponents {
                standard_v: 0,
                r:          Bytes::default(),
                s:          Bytes::default(),
            }),
            chain_id:  0u64,
            hash:      H256::default(),
        },
        sender,
        public: Some(Public::default()),
    }
}

#[async_trait]
impl SynchronizationAdapter for MockSyncAdapter {
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
    ) -> ProtocolResult<()> {
        Ok(())
    }

    /// Pull some blocks from other nodes from `begin` to `end`.
    async fn get_block_from_remote(
        &self,
        ctx: Context,
        number: BlockNumber,
    ) -> ProtocolResult<Block> {
        Ok(Block::default())
    }

    /// Pull signed transactions corresponding to the given hashes from other
    /// nodes.
    async fn get_txs_from_remote(
        &self,
        ctx: Context,
        number: BlockNumber,
        hashes: &[Hash],
    ) -> ProtocolResult<Vec<SignedTransaction>> {
        Ok(vec![])
    }

    async fn get_proof_from_remote(
        &self,
        ctx: Context,
        number: BlockNumber,
    ) -> ProtocolResult<Proof> {
        Ok(Proof::default())
    }

    fn get_tx_from_mem(&self, ctx: Context, tx_hash: &Hash) -> Option<SignedTransaction> {
        let tx = gen_tx(
            H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
            H160::from_str("0x1000000000000000000000000000000000000000").unwrap(),
            0,
            hex_decode("2839e92800000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000001").unwrap()
        );
        Some(tx)
    }
}

#[async_trait]
impl CommonConsensusAdapter for MockSyncAdapter {
    /// Save a block to the database.
    async fn save_block(&self, ctx: Context, block: Block) -> ProtocolResult<()> {
        Ok(())
    }

    async fn save_proof(&self, ctx: Context, proof: Proof) -> ProtocolResult<()> {
        Ok(())
    }

    /// Save some signed transactions to the database.
    async fn save_signed_txs(
        &self,
        ctx: Context,
        block_height: u64,
        signed_txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    async fn save_receipts(
        &self,
        ctx: Context,
        height: u64,
        receipts: Vec<Receipt>,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    /// Flush the given transactions in the mempool.
    async fn flush_mempool(
        &self,
        ctx: Context,
        ordered_tx_hashes: &[Hash],
        current_number: BlockNumber,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    /// Get a block corresponding to the given height.
    async fn get_block_by_number(&self, ctx: Context, height: u64) -> ProtocolResult<Block> {
        Ok(Block::default())
    }

    async fn get_block_header_by_number(
        &self,
        ctx: Context,
        height: u64,
    ) -> ProtocolResult<Header> {
        Ok(Header::default())
    }

    /// Get the current height from storage.
    async fn get_current_number(&self, ctx: Context) -> ProtocolResult<u64> {
        Ok(0)
    }

    async fn get_txs_from_storage(
        &self,
        ctx: Context,
        tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<SignedTransaction>> {
        Ok(vec![])
    }

    /// Execute some transactions.
    async fn exec(
        &self,
        ctx: Context,
        last_state_root: Hash,
        proposal: &Proposal,
        signed_txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<ExecResp> {
        Ok(ExecResp {
            state_root:   H256::from_str(
                "0xc2ca3b067635ecf9a5b17a398a2509a2bd93ed172bfb6699c7b046704ded529a",
            )
            .unwrap(),
            receipt_root: H256::from_str(
                "0xc2ca3b067635ecf9a5b17a398a2509a2bd93ed172bfb6699c7b046704ded529a",
            )
            .unwrap(),
            gas_used:     100,
            tx_resp:      vec![],
        })
    }

    fn need_change_metadata(&self, block_number: u64) -> bool {
        false
    }

    fn get_metadata_unchecked(&self, ctx: Context, block_number: u64) -> Metadata {
        Metadata::default()
    }

    fn get_metadata(&self, ctx: Context, header: &Header) -> ProtocolResult<Metadata> {
        Ok(Metadata::default())
    }

    fn update_metadata(&self, ctx: Context, header: &Header) -> ProtocolResult<()> {
        Ok(())
    }

    async fn broadcast_number(&self, ctx: Context, height: u64) -> ProtocolResult<()> {
        Ok(())
    }

    fn set_args(&self, context: Context, state_root: MerkleRoot, gas_limit: u64, max_tx_size: u64) {
    }

    fn tag_consensus(&self, ctx: Context, peer_ids: Vec<Bytes>) -> ProtocolResult<()> {
        Ok(())
    }

    async fn verify_proof(&self, ctx: Context, block: Block, proof: Proof) -> ProtocolResult<()> {
        Ok(())
    }

    async fn verify_block_header(&self, ctx: Context, block: &Proposal) -> ProtocolResult<()> {
        Ok(())
    }

    async fn notify_block_logs(
        &self,
        ctx: Context,
        block_number: u64,
        block_hash: Hash,
        logs: &[Vec<Log>],
    ) {
    }

    async fn notify_checkpoint(&self, ctx: Context, block: Block, proof: Proof) {}

    fn verify_proof_signature(
        &self,
        ctx: Context,
        block_height: u64,
        vote_hash: Bytes,
        aggregated_signature_bytes: Bytes,
        vote_pubkeys: Vec<Hex>,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    fn verify_proof_weight(
        &self,
        ctx: Context,
        block_height: u64,
        weight_map: HashMap<Bytes, u32>,
        signed_voters: Vec<Bytes>,
    ) -> ProtocolResult<()> {
        Ok(())
    }
}

// #[test]
#[tokio::test]
async fn test_new() {
    let sync_txs_chunk_size = 50;
    let status_agent = StatusAgent::new(CurrentStatus::default());
    let lock = Arc::new(AsyncMutex::new(()));

    // let network_service = mock_network_service();
    let consensus_adapter = MockSyncAdapter::default();
    let consensus_adapter = Arc::new(consensus_adapter);

    let synchronization = Arc::new(OverlordSynchronization::<_>::new(
        sync_txs_chunk_size,
        consensus_adapter,
        status_agent.clone(),
        lock,
    ));

    tokio::spawn(async move {
        if let Err(e) = synchronization.polling_broadcast().await {
            println!("synchronization: {:?}", e);
        }
    });

    println!("test_new end");
}

#[tokio::test]
async fn test_receive_remote_block() {
    let sync_txs_chunk_size = 50;
    let status_agent = StatusAgent::new(CurrentStatus::default());
    let lock = Arc::new(AsyncMutex::new(()));

    // let network_service = mock_network_service();
    let consensus_adapter = MockSyncAdapter::default();
    let consensus_adapter = Arc::new(consensus_adapter);

    let synchronization = Arc::new(OverlordSynchronization::<_>::new(
        sync_txs_chunk_size,
        consensus_adapter,
        status_agent.clone(),
        lock,
    ));

    let result = synchronization
        .receive_remote_block(Context::new(), 1)
        .await;
    assert!(result.is_err());
    println!("{:?}", result.err());
}
