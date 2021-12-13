mod engine;
mod synchronization;

use rand::random;

use protocol::types::{
    Address, Block, Bytes, Hash, Hasher, Header, Hex, MerkleRoot, Proof, Validator,
};

use crate::status::CurrentStatus;

const HEIGHT_TEN: u64 = 10;

fn mock_block_from_status(status: &CurrentStatus) -> Block {
    let block_header = Header {
        chain_id:          0,
        number:            status.last_number + 1,
        prev_hash:         status.prev_hash,
        timestamp:         random::<u64>(),
        transactions_root: mock_hash(),
        signed_txs_hash:   mock_hash(),
        state_root:        status.state_root,
        receipts_root:     status.receipts_root,
        gas_used:          status.gas_used,
        gas_limit:         status.gas_limit,
        proposer:          mock_address(),
        proof:             mock_proof(status.last_number),
        log_bloom:         Default::default(),
        difficulty:        Default::default(),
        extra_data:        Default::default(),
        mixed_hash:        Default::default(),
        nonce:             Default::default(),
        base_fee_per_gas:  Default::default(),
    };

    Block {
        header:    block_header,
        tx_hashes: vec![],
    }
}

fn mock_current_status() -> CurrentStatus {
    CurrentStatus {
        gas_used:         random::<u64>().into(),
        gas_limit:        random::<u64>().into(),
        log_bloom:        Default::default(),
        base_fee_per_gas: Default::default(),
        last_number:      HEIGHT_TEN,
        prev_hash:        mock_hash(),
        state_root:       mock_hash(),
        receipts_root:    mock_hash(),
        proof:            mock_proof(HEIGHT_TEN),
    }
}

fn mock_proof(proof_number: u64) -> Proof {
    Proof {
        number:     proof_number,
        round:      random::<u64>(),
        signature:  get_random_bytes(64),
        bitmap:     get_random_bytes(20),
        block_hash: mock_hash(),
    }
}

fn _mock_roots(len: u64) -> Vec<MerkleRoot> {
    (0..len).map(|_| mock_hash()).collect::<Vec<_>>()
}

fn mock_hash() -> Hash {
    Hasher::digest(get_random_bytes(10))
}

fn mock_address() -> Address {
    let hash = mock_hash();
    Address::from_slice(&hash.0[0..20])
}

fn get_random_bytes(len: usize) -> Bytes {
    let vec: Vec<u8> = (0..len).map(|_| random::<u8>()).collect();
    Bytes::from(vec)
}

fn mock_pub_key() -> Hex {
    Hex::from_string(
        "0x026c184a9016f6f71a234c86b141621f38b68c78602ab06768db4d83682c616004".to_owned(),
    )
    .unwrap()
}

fn _mock_validators(len: usize) -> Vec<Validator> {
    (0..len).map(|_| _mock_validator()).collect::<Vec<_>>()
}

fn _mock_validator() -> Validator {
    Validator {
        pub_key:        mock_pub_key().decode(),
        propose_weight: random::<u32>(),
        vote_weight:    random::<u32>(),
    }
}
