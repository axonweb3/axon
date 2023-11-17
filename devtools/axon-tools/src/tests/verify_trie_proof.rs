#[cfg(feature = "proof")]
use crate::verify_trie_proof;
#[cfg(feature = "proof")]
use eth_light_client_in_ckb_prover::Receipts;
#[cfg(feature = "proof")]
use ethereum_types::{H256, U256};
#[cfg(feature = "proof")]
use ethers_core::{
    types::{TransactionReceipt, U64},
    utils::rlp,
};

#[test]
#[cfg(all(feature = "proof", feature = "std"))]
fn test_verify_trie_proof() {
    let mut tx_receipts = Vec::<TransactionReceipt>::new();

    {
        let receipt = TransactionReceipt {
            transaction_hash: H256::from([0u8; 32]),
            transaction_index: 0.into(),
            cumulative_gas_used: U256::from(100),
            transaction_type: Some(U64::from(0)),
            status: Some(U64::from(1)),
            ..Default::default()
        };
        tx_receipts.push(receipt);
    }

    {
        let receipt = TransactionReceipt {
            transaction_hash: H256::from([1u8; 32]),
            transaction_index: 1.into(),
            gas_used: Some(U256::from(100)),
            transaction_type: Some(U64::from(1)),
            status: Some(U64::from(1)),
            ..Default::default()
        };
        tx_receipts.push(receipt);
    }

    let receipts: Receipts = tx_receipts.into();

    {
        println!("proof of index 0");
        let proof_index = 0u64;
        let receipt_proof = receipts.generate_proof(proof_index as usize);

        {
            println!("test key 0");
            let key = rlp::encode(&proof_index);
            let result = verify_trie_proof(receipts.root(), &key, receipt_proof.clone());
            assert!(result.unwrap().is_some());
        }

        {
            println!("test key 1");
            let key = rlp::encode(&(1u64));
            let result = verify_trie_proof(receipts.root(), &key, receipt_proof.clone());
            assert!(result.unwrap().is_none());
        }

        {
            println!("test key 2");
            let key = rlp::encode(&(2u64));
            let result = verify_trie_proof(receipts.root(), &key, receipt_proof.clone());
            assert!(result.unwrap().is_none());
        }

        {
            println!("test illegal trie root");
            let key = rlp::encode(&(200u64));
            let result = verify_trie_proof(H256::from([4u8; 32]), &key, receipt_proof.clone());
            assert!(result.is_err());
        }
    }

    {
        println!("proof of index 1, wrong");
        let proof_index = 1u64;
        let receipt_proof = receipts.generate_proof(proof_index as usize);

        {
            println!("test key 0");
            let key = rlp::encode(&(0u64));
            let result = verify_trie_proof(receipts.root(), &key, receipt_proof.clone());
            assert!(result.unwrap().is_none());
        }

        {
            println!("test key 1");
            let key = rlp::encode(&(1u64));
            let result = verify_trie_proof(receipts.root(), &key, receipt_proof.clone());
            assert!(result.unwrap().is_some());
        }
    }
}
