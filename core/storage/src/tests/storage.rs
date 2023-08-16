use std::sync::Arc;

use protocol::traits::{Context, ReadOnlyStorage, Storage};
use protocol::types::Hasher;

use core_db::MemoryAdapter;

use crate::tests::{get_random_bytes, mock_block, mock_proof, mock_receipt, mock_signed_tx};
use crate::ImplStorage;

macro_rules! exec {
    ($func: expr) => {
        futures::executor::block_on(async { $func.await.unwrap() })
    };
}

#[test]
fn test_storage_block_insert() {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);

    let height = 100;
    let block = mock_block(height, Hasher::digest(get_random_bytes(10)));
    let block_hash = block.hash();

    exec!(storage.insert_block(Context::new(), block));

    let block = exec!(storage.get_latest_block(Context::new()));
    assert_eq!(height, block.header.number);

    let block = exec!(storage.get_block(Context::new(), height));
    assert_eq!(Some(height), block.map(|b| b.header.number));

    let block = exec!(storage.get_block_by_hash(Context::new(), &block_hash));
    assert_eq!(height, block.unwrap().header.number);
}

#[test]
fn test_storage_receipts_insert() {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let height = 2077;

    let mut receipts = Vec::new();
    let mut hashes = Vec::new();

    for _ in 0..1 {
        let hash = Hasher::digest(get_random_bytes(10));
        hashes.push(hash);
        let receipt = mock_receipt(hash);
        receipts.push(receipt);
    }

    exec!(storage.insert_receipts(Context::new(), height, receipts.clone()));
    let receipts_2 = exec!(storage.get_receipts(Context::new(), height, &hashes));

    for i in 0..1 {
        assert_eq!(
            Some(receipts.get(i).unwrap()),
            receipts_2.get(i).unwrap().as_ref()
        );
    }
}

#[test]
fn test_storage_transactions_insert() {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let height = 2020;

    let mut transactions = Vec::new();
    let mut hashes = Vec::new();

    for _ in 0..10 {
        let transaction = mock_signed_tx();
        hashes.push(transaction.transaction.hash);
        transactions.push(transaction);
    }

    exec!(storage.insert_transactions(Context::new(), height, transactions.clone()));
    let transactions_2 = exec!(storage.get_transactions(Context::new(), height, &hashes));

    for i in 0..10 {
        assert_eq!(
            Some(transactions.get(i).unwrap()),
            transactions_2.get(i).unwrap().as_ref()
        );
    }
}

#[test]
fn test_storage_latest_proof_insert() {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);

    let block_hash = Hasher::digest(get_random_bytes(10));
    let proof = mock_proof(block_hash);

    exec!(storage.update_latest_proof(Context::new(), proof.clone()));
    let proof_2 = exec!(storage.get_latest_proof(Context::new(),));

    assert_eq!(proof.block_hash, proof_2.block_hash);
}

#[test]
fn test_storage_evm_code_insert() {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);

    let code = get_random_bytes(1000);
    let code_hash = Hasher::digest(&code);
    let address = Hasher::digest(code_hash);

    exec!(storage.insert_code(Context::new(), address, code_hash, code.clone()));

    let code_2 = exec!(storage.get_code_by_hash(Context::new(), &code_hash));
    assert_eq!(code, code_2.unwrap());

    let code_3 = exec!(storage.get_code_by_address(Context::new(), &address));
    assert_eq!(code, code_3.unwrap());
}
