mod engine;
pub mod synchronization;

use rand::random;

use protocol::types::{
    Address, Block, Bytes, Hash, Hasher, Header, Hex, MerkleRoot, Proof, Validator, U256,
};

use crate::status::CurrentStatus;
use crate::OverlordSynchronization;

const _HEIGHT_TEN: u64 = 10;

fn _mock_block_from_status(status: &CurrentStatus) -> Block {
    let block_header = Header {
        chain_id:                   0,
        number:                     status.last_number + 1,
        prev_hash:                  status.prev_hash,
        timestamp:                  random::<u64>(),
        transactions_root:          _mock_hash(),
        signed_txs_hash:            _mock_hash(),
        state_root:                 Default::default(),
        receipts_root:              Default::default(),
        gas_used:                   Default::default(),
        gas_limit:                  Default::default(),
        proposer:                   _mock_address().0,
        proof:                      _mock_proof(status.last_number),
        log_bloom:                  Default::default(),
        difficulty:                 Default::default(),
        extra_data:                 Default::default(),
        mixed_hash:                 Default::default(),
        nonce:                      Default::default(),
        base_fee_per_gas:           Default::default(),
        last_checkpoint_block_hash: Default::default(),
        call_system_script_count:   0,
    };

    Block {
        header:    block_header,
        tx_hashes: vec![],
    }
}

fn _mock_current_status() -> CurrentStatus {
    CurrentStatus {
        prev_hash:                  _mock_hash(),
        last_number:                0,
        last_state_root:            _mock_hash(),
        tx_num_limit:               9,
        max_tx_size:                U256::zero(),
        proof:                      Proof::default(),
        last_checkpoint_block_hash: _mock_hash(),
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
        pub_key:        _mock_pub_key().as_bytes(),
        propose_weight: random::<u32>(),
        vote_weight:    random::<u32>(),
    }
}
