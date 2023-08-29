#![allow(unused_variables)]

// mod engine;
pub mod synchronization;

use std::{collections::HashMap, str::FromStr};

use protocol::rand::random;
use protocol::{
    async_trait,
    codec::hex_decode,
    traits::{CommonConsensusAdapter, Context, SynchronizationAdapter},
    types::{
        Address, Block, BlockNumber, Bytes, Eip1559Transaction, ExecResp, Hash, Hasher, Header,
        Hex, MerkleRoot, Metadata, Proof, Proposal, Public, Receipt, SignatureComponents,
        SignedTransaction, TransactionAction, UnsignedTransaction, UnverifiedTransaction,
        Validator, H160, H256, U256,
    },
    ProtocolResult,
};

use crate::status::CurrentStatus;

const _HEIGHT_TEN: u64 = 10;

fn _mock_block_from_status(status: &CurrentStatus) -> Block {
    let block_header = Header {
        version:                  Default::default(),
        chain_id:                 0,
        number:                   status.last_number + 1,
        prev_hash:                status.prev_hash,
        timestamp:                random::<u64>(),
        transactions_root:        _mock_hash(),
        signed_txs_hash:          _mock_hash(),
        state_root:               Default::default(),
        receipts_root:            Default::default(),
        gas_used:                 Default::default(),
        gas_limit:                Default::default(),
        proposer:                 _mock_address().0,
        proof:                    _mock_proof(status.last_number),
        log_bloom:                Default::default(),
        extra_data:               Default::default(),
        base_fee_per_gas:         Default::default(),
        call_system_script_count: 0,
    };

    Block {
        header:    block_header,
        tx_hashes: vec![],
    }
}

fn _mock_current_status() -> CurrentStatus {
    CurrentStatus {
        prev_hash:       _mock_hash(),
        last_number:     0,
        last_state_root: _mock_hash(),
        tx_num_limit:    9,
        max_tx_size:     U256::zero(),
        proof:           Proof::default(),
    }
}

fn _mock_proof(proof_number: u64) -> Proof {
    Proof {
        number:     proof_number,
        round:      random::<u64>(),
        signature:  _get_random_bytes(64),
        bitmap:     _get_random_bytes(20),
        block_hash: _mock_hash(),
    }
}

fn _mock_roots(len: u64) -> Vec<MerkleRoot> {
    (0..len).map(|_| _mock_hash()).collect::<Vec<_>>()
}

fn _mock_hash() -> Hash {
    Hasher::digest(_get_random_bytes(10))
}

fn _mock_address() -> Address {
    let hash = _mock_hash();
    Address::from_hash(hash)
}

fn _get_random_bytes(len: usize) -> Bytes {
    let vec: Vec<u8> = (0..len).map(|_| random::<u8>()).collect();
    Bytes::from(vec)
}

fn _mock_pub_key() -> Hex {
    Hex::from_str("0x026c184a9016f6f71a234c86b141621f38b68c78602ab06768db4d83682c616004").unwrap()
}

fn _mock_validators(len: usize) -> Vec<Validator> {
    (0..len).map(|_| _mock_validator()).collect::<Vec<_>>()
}

fn _mock_validator() -> Validator {
    Validator {
        pub_key:        _mock_pub_key().as_bytes(),
        propose_weight: random::<u32>(),
        vote_weight:    random::<u32>(),
    }
}

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
            chain_id:  Some(0u64),
            hash:      H256::default(),
        },
        sender,
        public: Some(Public::default()),
    }
}

#[derive(Debug, Default)]
pub struct MockSyncAdapter {}

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
        signed_txs: &[SignedTransaction],
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

    async fn is_last_block_in_current_epoch(&self, block_number: u64) -> ProtocolResult<bool> {
        Ok(false)
    }

    async fn get_metadata_by_block_number(&self, block_number: u64) -> ProtocolResult<Metadata> {
        Ok(Metadata::default())
    }

    async fn get_metadata_by_epoch(&self, epoch: u64) -> ProtocolResult<Metadata> {
        Ok(Metadata::default())
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

    async fn verify_proof_signature(
        &self,
        ctx: Context,
        block_height: u64,
        vote_hash: Bytes,
        aggregated_signature_bytes: Bytes,
        vote_pubkeys: Vec<Hex>,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    async fn verify_proof_weight(
        &self,
        ctx: Context,
        block_height: u64,
        weight_map: HashMap<Bytes, u32>,
        signed_voters: Vec<Bytes>,
    ) -> ProtocolResult<()> {
        Ok(())
    }
}
