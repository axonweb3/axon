use criterion::{criterion_group, criterion_main, Criterion};

use common_crypto::{
    Crypto, PrivateKey, Secp256k1Recoverable, Secp256k1RecoverablePrivateKey, Signature,
};
use core_consensus::SignedTxsWAL;
use protocol::rand::{random, rngs::OsRng};
use protocol::types::{
    Bytes, Eip1559Transaction, Hash, Hasher, SignatureComponents, SignedTransaction,
    TransactionAction, UnsignedTransaction, UnverifiedTransaction,
};

static FULL_TXS_PATH: &str = "./free-space/wal/txs";

fn mock_wal_txs(size: usize) -> Vec<SignedTransaction> {
    (0..size).map(|_| mock_sign_tx()).collect::<Vec<_>>()
}

fn mock_hash() -> Hash {
    Hash::random()
}

pub fn get_random_bytes(len: usize) -> Bytes {
    let vec: Vec<u8> = (0..len).map(|_| random::<u8>()).collect();
    Bytes::from(vec)
}

fn mock_sign_tx() -> SignedTransaction {
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
        hash:      mock_hash(),
    }
    .calc_hash();

    let priv_key = Secp256k1RecoverablePrivateKey::generate(&mut OsRng);
    let signature = Secp256k1Recoverable::sign_message(utx.hash.as_bytes(), &priv_key.to_bytes())
        .unwrap()
        .to_bytes();
    utx.signature = Some(signature.into());

    SignedTransaction::from_unverified(utx).unwrap()
}

fn criterion_save_wal(c: &mut Criterion) {
    // MacOS M1 Pro, 16GB: time: 933.60µs
    c.bench_function("save wal 1000 txs", |b| {
        let wal = SignedTxsWAL::new(FULL_TXS_PATH);
        let txs = mock_wal_txs(1000);
        let txs_hash = Hasher::digest(Bytes::from(rlp::encode_list(&txs)));

        b.iter(|| {
            wal.save(1u64, txs_hash, txs.clone()).unwrap();
        })
    });
    // MacOS M1 Pro, 16GB: time: 1.80ms
    c.bench_function("save wal 2000 txs", |b| {
        let wal = SignedTxsWAL::new(FULL_TXS_PATH);
        let txs = mock_wal_txs(2000);
        let txs_hash = Hasher::digest(Bytes::from(rlp::encode_list(&txs)));

        b.iter(|| {
            wal.save(1u64, txs_hash, txs.clone()).unwrap();
        })
    });
    // MacOS M1 Pro, 16GB: time: 3.68ms
    c.bench_function("save wal 4000 txs", |b| {
        let wal = SignedTxsWAL::new(FULL_TXS_PATH);
        let txs = mock_wal_txs(4000);
        let txs_hash = Hasher::digest(Bytes::from(rlp::encode_list(&txs)));

        b.iter(|| {
            wal.save(1u64, txs_hash, txs.clone()).unwrap();
        })
    });
    // MacOS M1 Pro, 16GB: time: 7.28ms
    c.bench_function("save wal 8000 txs", |b| {
        let wal = SignedTxsWAL::new(FULL_TXS_PATH);
        let txs = mock_wal_txs(8000);
        let txs_hash = Hasher::digest(Bytes::from(rlp::encode_list(&txs)));

        b.iter(|| {
            wal.save(1u64, txs_hash, txs.clone()).unwrap();
        })
    });
    // MacOS M1 Pro, 16GB: time: 15.33ms
    c.bench_function("save wal 16000 txs", |b| {
        let wal = SignedTxsWAL::new(FULL_TXS_PATH);
        let txs = mock_wal_txs(16000);
        let txs_hash = Hasher::digest(Bytes::from(rlp::encode_list(&txs)));

        b.iter(|| {
            wal.save(1u64, txs_hash, txs.clone()).unwrap();
        })
    });
}

fn criterion_txs_rlp_encode(c: &mut Criterion) {
    // MacOS M1 Pro, 16GB: time: 16.57ms
    c.bench_function("rlp encode 20000 txs", |b| {
        let txs = mock_wal_txs(20000);

        b.iter(|| {
            let _ = rlp::encode_list(&txs);
        });
    });
}

criterion_group!(benches, criterion_save_wal, criterion_txs_rlp_encode);
criterion_main!(benches);
