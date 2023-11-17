#[cfg(feature = "proof")]
use crate::hash::keccak_256;
#[cfg(feature = "proof")]
use crate::types::{Block, BlockVersion, Metadata, Proof, Proposal, ValidatorExtend};
#[cfg(feature = "proof")]
use bytes::Bytes;
#[cfg(feature = "proof")]
use ethereum_types::{H160, H256, U256};
#[cfg(feature = "proof")]
use rlp::Encodable;
#[cfg(feature = "proof")]
use serde::de::DeserializeOwned;

#[cfg(feature = "proof")]
fn read_json<T: DeserializeOwned>(path: &str) -> T {
    let json = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
#[cfg(feature = "proof")]
fn test_proposal() {
    let proposal = Proposal {
        version:                  BlockVersion::V0,
        prev_hash:                H256::from([1u8; 32]),
        proposer:                 H160::from([2u8; 20]),
        prev_state_root:          H256::from([3u8; 32]),
        transactions_root:        H256::from([4u8; 32]),
        signed_txs_hash:          H256::from([5u8; 32]),
        timestamp:                0,
        number:                   100,
        gas_limit:                U256::from(6),
        extra_data:               Vec::new(),
        base_fee_per_gas:         U256::from(7),
        proof:                    Proof {
            number:     0,
            round:      1,
            block_hash: H256::from([1u8; 32]),
            signature:  Bytes::from("1234"),
            bitmap:     Bytes::from("abcd"),
        },
        chain_id:                 1000,
        call_system_script_count: 1,
        tx_hashes:                vec![],
    };

    let rlp_bytes = proposal.rlp_bytes();
    let hash = keccak_256(&rlp_bytes);
    let ref_hash = [
        95u8, 14, 98, 221, 57, 150, 113, 31, 144, 120, 33, 170, 102, 71, 19, 108, 141, 247, 85, 17,
        193, 76, 201, 242, 128, 231, 44, 231, 204, 189, 67, 230,
    ];
    assert_eq!(hash, ref_hash);
}

#[test]
#[cfg(feature = "proof")]
fn test_verify_proof() {
    let block: Block = read_json("src/tests/block.json");
    let proof: Proof = read_json("src/tests/proof.json");
    let metadata: Metadata = read_json("src/tests/metadata.json");
    let mut validators = metadata
        .verifier_list
        .iter()
        .map(|v| ValidatorExtend {
            bls_pub_key:    v.bls_pub_key.clone(),
            pub_key:        v.pub_key.clone(),
            address:        v.address,
            propose_weight: v.propose_weight,
            vote_weight:    v.vote_weight,
        })
        .collect::<Vec<_>>();

    let previous_state_root =
        hex::decode("9fc948be2cfb0127e979dc9c7e6d2f4a2890b54e0e81fd69c687303e6b25ddde").unwrap();

    let result = crate::verify_proof(
        block,
        H256::from_slice(&previous_state_root),
        &mut validators,
        proof,
    );

    assert!(result.is_ok());
}
