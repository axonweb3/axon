use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;

use overlord::types::{AggregatedSignature, Commit, Proof as OverlordProof};

use common_crypto::BlsPrivateKey;
use protocol::codec::ProtocolCodec;
use protocol::traits::{
    CommonConsensusAdapter, ConsensusAdapter, Context, MessageTarget, NodeInfo,
};
use protocol::types::{
    Block, Bytes, ExecResp, Hash, Hasher, Header, Hex, Metadata, MetadataVersion, Pill, Proof,
    Receipt, SignedTransaction, H256,
};
use protocol::{async_trait, tokio::sync::Mutex, ProtocolResult};

use crate::engine::ConsensusEngine;
use crate::status::StatusAgent;
use crate::util::OverlordCrypto;
use crate::wal::{ConsensusWal, SignedTxsWAL};

use super::*;

static _FULL_TXS_PATH: &str = "./free-space/engine/txs";
static _FULL_CONSENSUS_PATH: &str = "./free-space/engine/consensus";

fn _mock_commit(block: Block) -> Commit<Pill> {
    let pill = Pill {
        block:          block.clone(),
        propose_hashes: vec![],
    };
    Commit {
        height:  11,
        content: pill,
        proof:   OverlordProof {
            height:     11,
            round:      0,
            block_hash: Bytes::from(
                Hasher::digest(block.header.encode().unwrap())
                    .as_bytes()
                    .to_vec(),
            ),
            signature:  AggregatedSignature {
                signature:      _get_random_bytes(32),
                address_bitmap: _get_random_bytes(10),
            },
        },
    }
}

fn _init_engine(init_status: CurrentStatus) -> ConsensusEngine<MockConsensusAdapter> {
    ConsensusEngine::new(
        StatusAgent::new(init_status),
        _mock_node_info(),
        Arc::new(SignedTxsWAL::new(_FULL_TXS_PATH)),
        Arc::new(MockConsensusAdapter {}),
        Arc::new(_init_crypto()),
        Arc::new(Mutex::new(())),
        Arc::new(ConsensusWal::new(_FULL_CONSENSUS_PATH)),
    )
}

fn _init_crypto() -> OverlordCrypto {
    let mut priv_key = Vec::new();
    priv_key.extend_from_slice(&[0u8; 16]);
    let mut tmp =
        hex::decode("45c56be699dca666191ad3446897e0f480da234da896270202514a0e1a587c3f").unwrap();
    priv_key.append(&mut tmp);

    OverlordCrypto::new(
        BlsPrivateKey::try_from(priv_key.as_ref()).unwrap(),
        HashMap::new(),
        std::str::from_utf8(hex::decode("").unwrap().as_ref())
            .unwrap()
            .into(),
    )
}

fn _mock_node_info() -> NodeInfo {
    NodeInfo {
        self_pub_key: _mock_pub_key().as_bytes(),
        chain_id:     0,
        self_address: _mock_address(),
    }
}

fn _mock_metadata() -> Metadata {
    Metadata {
        version:                    MetadataVersion::new(0, 100000),
        common_ref:                 Hex::from_string("0x703873635a6b51513451".to_string()).unwrap(),
        timeout_gap:                20,
        gas_limit:                  600000,
        gas_price:                  1,
        interval:                   3000,
        verifier_list:              vec![],
        propose_ratio:              3,
        prevote_ratio:              3,
        precommit_ratio:            3,
        brake_ratio:                3,
        tx_num_limit:               3,
        max_tx_size:                3000,
        last_checkpoint_block_hash: Default::default(),
    }
}

pub struct MockConsensusAdapter;

#[async_trait]
impl CommonConsensusAdapter for MockConsensusAdapter {
    async fn save_block(&self, _ctx: Context, _block: Block) -> ProtocolResult<()> {
        Ok(())
    }

    async fn save_proof(&self, _ctx: Context, _proof: Proof) -> ProtocolResult<()> {
        Ok(())
    }

    async fn save_signed_txs(
        &self,
        _ctx: Context,
        _block_number: u64,
        _signed_txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    async fn save_receipts(
        &self,
        _ctx: Context,
        _number: u64,
        _receipts: Vec<Receipt>,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    async fn flush_mempool(
        &self,
        _ctx: Context,
        _ordered_tx_hashes: &[Hash],
    ) -> ProtocolResult<()> {
        Ok(())
    }

    async fn get_block_by_number(&self, _ctx: Context, _number: u64) -> ProtocolResult<Block> {
        unimplemented!()
    }

    async fn get_block_header_by_number(
        &self,
        _ctx: Context,
        _number: u64,
    ) -> ProtocolResult<Header> {
        unimplemented!()
    }

    async fn get_current_number(&self, _ctx: Context) -> ProtocolResult<u64> {
        Ok(10)
    }

    async fn get_txs_from_storage(
        &self,
        _ctx: Context,
        _tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<SignedTransaction>> {
        unimplemented!()
    }

    async fn verify_block_header(&self, _ctx: Context, _block: &Block) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn exec(
        &self,
        _ctx: Context,
        _block_hash: Hash,
        _header: &Header,
        _signed_txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<ExecResp> {
        unimplemented!()
    }

    async fn verify_proof(
        &self,
        _ctx: Context,
        _block_header: &Header,
        _proof: Proof,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    async fn broadcast_number(&self, _ctx: Context, _number: u64) -> ProtocolResult<()> {
        Ok(())
    }

    fn set_args(
        &self,
        _context: Context,
        _state_root: H256,
        _timeout_gap: u64,
        _cycles_limit: u64,
        _max_tx_size: u64,
    ) {
    }

    fn tag_consensus(&self, _ctx: Context, _peer_ids: Vec<Bytes>) -> ProtocolResult<()> {
        Ok(())
    }

    fn verify_proof_signature(
        &self,
        _ctx: Context,
        _block_number: u64,
        _vote_hash: Bytes,
        _aggregated_signature_bytes: Bytes,
        _vote_pubkeys: Vec<Hex>,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    fn verify_proof_weight(
        &self,
        _ctx: Context,
        _block_number: u64,
        _weight_map: HashMap<Bytes, u32>,
        _signed_voters: Vec<Bytes>,
    ) -> ProtocolResult<()> {
        Ok(())
    }
}

#[async_trait]
impl ConsensusAdapter for MockConsensusAdapter {
    async fn get_txs_from_mempool(
        &self,
        _ctx: Context,
        _number: u64,
        _cycles_limit: u64,
        _tx_num_limit: u64,
    ) -> ProtocolResult<Vec<Hash>> {
        unimplemented!()
    }

    async fn sync_txs(&self, _ctx: Context, _txs: Vec<Hash>) -> ProtocolResult<()> {
        Ok(())
    }

    async fn get_full_txs(
        &self,
        _ctx: Context,
        _txs: &[Hash],
    ) -> ProtocolResult<Vec<SignedTransaction>> {
        Ok(vec![])
    }

    async fn transmit(
        &self,
        _ctx: Context,
        _msg: Vec<u8>,
        _end: &str,
        _target: MessageTarget,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    async fn pull_block(&self, _ctx: Context, _number: u64, _end: &str) -> ProtocolResult<Block> {
        unimplemented!()
    }

    async fn get_current_number(&self, _ctx: Context) -> ProtocolResult<u64> {
        Ok(10)
    }

    async fn verify_txs(&self, _ctx: Context, _number: u64, _txs: &[Hash]) -> ProtocolResult<()> {
        Ok(())
    }
}
