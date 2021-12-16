use protocol::traits::{StorageAdapter, StorageBatchModify};

use crate::adapter::memory::MemoryAdapter;
use crate::adapter::rocks::RocksAdapter;
use crate::tests::mock_signed_tx;
use crate::{CommonHashKey, TransactionSchema};

#[test]
fn test_adapter_insert() {
    adapter_insert_test(MemoryAdapter::new());
    adapter_insert_test(RocksAdapter::new("rocksdb/test_adapter_insert".to_string(), 64).unwrap())
}

#[test]
fn test_adapter_batch_modify() {
    adapter_batch_modify_test(MemoryAdapter::new());
    adapter_batch_modify_test(
        RocksAdapter::new("rocksdb/test_adapter_batch_modify".to_string(), 64).unwrap(),
    )
}

#[test]
fn test_adapter_remove() {
    adapter_remove_test(MemoryAdapter::new());
    adapter_remove_test(RocksAdapter::new("rocksdb/test_adapter_remove".to_string(), 64).unwrap())
}

fn adapter_insert_test(db: impl StorageAdapter) {
    let stx = mock_signed_tx();
    let tx_key = CommonHashKey::new(1, stx.transaction.hash);
    exec!(db.insert::<TransactionSchema>(tx_key.clone(), stx.clone()));
    let stx = exec!(db.get::<TransactionSchema>(tx_key)).unwrap();

    assert!(stx.transaction.check_hash());
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

    exec!(db.batch_modify::<TransactionSchema>(keys.clone(), inserts));
    let opt_stxs = exec!(db.get_batch::<TransactionSchema>(keys));

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
    exec!(db.insert::<TransactionSchema>(tx_key.clone(), stx.clone()));
    let is_exist = exec!(db.contains::<TransactionSchema>(tx_key.clone()));
    assert!(is_exist);

    exec!(db.remove::<TransactionSchema>(tx_key.clone()));
    let is_exist = exec!(db.contains::<TransactionSchema>(tx_key));
    assert!(!is_exist);
}
