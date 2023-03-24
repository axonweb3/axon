use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};

use common_crypto::{
    Crypto, PrivateKey, Secp256k1Recoverable, Secp256k1RecoverablePrivateKey, Signature,
};
use core_storage::ImplStorage;
use protocol::rand::{random, rngs::OsRng};
use protocol::traits::{Context, Storage};
use protocol::types::{
    Bytes, Eip1559Transaction, ExitReason, ExitSucceed, Hash, Hasher, Receipt, SignatureComponents,
    SignedTransaction, TransactionAction, UnsignedTransaction, UnverifiedTransaction,
};

use core_storage::adapter::memory::MemoryAdapter;

macro_rules! exec {
    ($func: expr) => {
        futures::executor::block_on(async { $func.await.unwrap() })
    };
}

fn criterion_insert_txs(c: &mut Criterion) {
    // MacOS M1 Pro, 16GB: time: 19.939ms
    c.bench_function("insert 10000 txs", |b| {
        let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
        let height = 2077;

        let txs = (0..10000).map(|_| mock_signed_tx()).collect::<Vec<_>>();

        b.iter(|| {
            exec!(storage.insert_transactions(Context::new(), height, txs.clone()));
        })
    });
    // MacOS M1 Pro, 16GB: time: 39.443ms
    c.bench_function("insert 20000 txs", |b| {
        let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
        let height = 2077;

        let txs = (0..20000).map(|_| mock_signed_tx()).collect::<Vec<_>>();

        b.iter(|| {
            exec!(storage.insert_transactions(Context::new(), height, txs.clone()));
        })
    });
    // MacOS M1 Pro, 16GB: time: 81.752ms
    c.bench_function("insert 40000 txs", |b| {
        let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
        let height = 2077;

        let txs = (0..40000).map(|_| mock_signed_tx()).collect::<Vec<_>>();

        b.iter(|| {
            exec!(storage.insert_transactions(Context::new(), height, txs.clone()));
        })
    });
}

fn criterion_insert_receipts(c: &mut Criterion) {
    // MacOS M1 Pro, 16GB: time: 27.191ms
    c.bench_function("insert 10000 receipts", |b| {
        let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
        let height = 2045;

        let receipts = (0..10000)
            .map(|_| mock_receipt(Hasher::digest(get_random_bytes(10))))
            .collect::<Vec<_>>();

        b.iter(|| {
            exec!(storage.insert_receipts(Context::new(), height, receipts.clone()));
        })
    });
    // MacOS M1 Pro, 16GB: time: 54.249ms
    c.bench_function("insert 20000 receipts", |b| {
        let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
        let height = 2045;

        let receipts = (0..20000)
            .map(|_| mock_receipt(Hasher::digest(get_random_bytes(10))))
            .collect::<Vec<_>>();

        b.iter(|| {
            exec!(storage.insert_receipts(Context::new(), height, receipts.clone()));
        })
    });
    // MacOS M1 Pro, 16GB: time: 110.05ms
    c.bench_function("insert 40000 receipts", |b| {
        let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
        let height = 2077;

        let receipts = (0..40000)
            .map(|_| mock_receipt(Hasher::digest(get_random_bytes(10))))
            .collect::<Vec<_>>();

        b.iter(|| {
            exec!(storage.insert_receipts(Context::new(), height, receipts.clone()));
        })
    });
    // MacOS M1 Pro, 16GB: time: 225.67ms
    c.bench_function("insert 80000 receipts", |b| {
        let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
        let height = 2077;

        let receipts = (0..80000)
            .map(|_| mock_receipt(Hasher::digest(get_random_bytes(10))))
            .collect::<Vec<_>>();

        b.iter(|| {
            exec!(storage.insert_receipts(Context::new(), height, receipts.clone()));
        })
    });
}

fn mock_signed_tx() -> SignedTransaction {
    let mut utx = UnverifiedTransaction {
        unsigned:  UnsignedTransaction::Eip1559(Eip1559Transaction {
            nonce:                    Default::default(),
            max_priority_fee_per_gas: Default::default(),
            gas_price:                Default::default(),
            gas_limit:                Default::default(),
            action:                   TransactionAction::Create,
            value:                    Default::default(),
            data:                     Bytes::new(),
            access_list:              vec![],
        }),
        signature: Some(SignatureComponents {
            standard_v: 4,
            r:          Default::default(),
            s:          Default::default(),
        }),
        chain_id:  Some(random::<u64>()),
        hash:      Default::default(),
    }
    .calc_hash();

    let priv_key = Secp256k1RecoverablePrivateKey::generate(&mut OsRng);
    let signature = Secp256k1Recoverable::sign_message(
        utx.signature_hash(true).as_bytes(),
        &priv_key.to_bytes(),
    )
    .unwrap()
    .to_bytes();
    utx.signature = Some(signature.into());

    utx.try_into().unwrap()
}

fn get_random_bytes(len: usize) -> Bytes {
    let vec: Vec<u8> = (0..len).map(|_| random::<u8>()).collect();
    Bytes::from(vec)
}

fn mock_receipt(hash: Hash) -> Receipt {
    Receipt {
        tx_hash:      hash,
        block_number: random::<u64>(),
        block_hash:   Default::default(),
        tx_index:     random::<u32>(),
        state_root:   Default::default(),
        used_gas:     Default::default(),
        logs_bloom:   Default::default(),
        logs:         vec![],
        log_index:    1,
        code_address: None,
        sender:       Default::default(),
        ret:          ExitReason::Succeed(ExitSucceed::Stopped),
        removed:      false,
    }
}

criterion_group!(benches, criterion_insert_txs, criterion_insert_receipts);
criterion_main!(benches);
