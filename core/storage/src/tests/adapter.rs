use core_db::{MemoryAdapter, RocksAdapter};
use protocol::traits::{StorageAdapter, StorageBatchModify};

use crate::{tests::mock_signed_tx, CommonHashKey, TransactionSchema};

#[test]
fn test_adapter_insert() {
    adapter_insert_test(MemoryAdapter::new());
    adapter_insert_test(
        RocksAdapter::new("rocksdb/test_adapter_insert", Default::default()).unwrap(),
    )
}

#[test]
fn test_adapter_batch_modify() {
    adapter_batch_modify_test(MemoryAdapter::new());
    adapter_batch_modify_test(
        RocksAdapter::new("rocksdb/test_adapter_batch_modify", Default::default()).unwrap(),
    )
}

#[test]
fn test_adapter_remove() {
    adapter_remove_test(MemoryAdapter::new());
    adapter_remove_test(
        RocksAdapter::new("rocksdb/test_adapter_remove", Default::default()).unwrap(),
    )
}

fn adapter_insert_test(db: impl StorageAdapter) {
    let stx = mock_signed_tx();
    let tx_key = CommonHashKey::new(1, stx.transaction.hash);
    db.insert::<TransactionSchema>(tx_key.clone(), stx).unwrap();
    let stx = db.get::<TransactionSchema>(tx_key).unwrap().unwrap();

    assert!(stx.transaction.check_hash().is_ok());
}

fn adapter_batch_modify_test(db: impl StorageAdapter) {
    let mut stxs = Vec::new();
    let mut keys = Vec::new();
    let mut inserts = Vec::new();

    for _ in 0..10 {
        let stx = mock_signed_tx();
        keys.push(CommonHashKey::new(1, stx.transaction.hash));
        stxs.push(stx.clone());
        inserts.push(StorageBatchModify::Insert::<TransactionSchema>(stx));
    }

    db.batch_modify::<TransactionSchema>(keys.clone(), inserts)
        .unwrap();
    let opt_stxs = db.get_batch::<TransactionSchema>(keys).unwrap();

    for i in 0..10 {
        assert_eq!(
            stxs.get(i).unwrap().transaction.hash,
            opt_stxs.get(i).unwrap().as_ref().unwrap().transaction.hash
        );
    }
}

fn adapter_remove_test(db: impl StorageAdapter) {
    let stx = mock_signed_tx();
    let tx_key = CommonHashKey::new(1, stx.transaction.hash);
    db.insert::<TransactionSchema>(tx_key.clone(), stx).unwrap();
    let is_exist = db.contains::<TransactionSchema>(tx_key.clone()).unwrap();
    assert!(is_exist);

    db.remove::<TransactionSchema>(tx_key.clone()).unwrap();
    let is_exist = db.contains::<TransactionSchema>(tx_key).unwrap();
    assert!(!is_exist);
}
