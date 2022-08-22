use std::convert::TryFrom;

use criterion::{criterion_group, criterion_main, Criterion};
use overlord::types::{Vote, VoteType};

use common_crypto::*;
use protocol::types::{Bytes, Hasher, H256};

fn criterion_aggregated_sig(c: &mut Criterion) {
    // MacOS M1 Pro 16GB: 185.68 µs
    c.bench_function("4 aggregated sig", |b| {
        let vote_msg =
            HashValue::try_from(Hasher::digest(Bytes::from(rlp::encode(&mock_vote()))).as_bytes())
                .unwrap();

        let mut priv_pub_keys = Vec::new();
        let mut signatures = Vec::new();
        gen_key_pair_sigs(4, &mut priv_pub_keys, &mut signatures, &vote_msg);

        let sigs_pubkeys = signatures
            .iter()
            .zip(priv_pub_keys.iter())
            .map(|(sig, key_pair)| (sig.clone(), key_pair.1.clone()))
            .collect::<Vec<_>>();
        b.iter(|| {
            let _ = BlsSignature::combine(sigs_pubkeys.clone());
        })
    });
    // MacOS M1 Pro 16GB: 364.47 µs
    c.bench_function("8 aggregated sig", |b| {
        let vote_msg =
            HashValue::try_from(Hasher::digest(Bytes::from(rlp::encode(&mock_vote()))).as_bytes())
                .unwrap();

        let mut priv_pub_keys = Vec::new();
        let mut signatures = Vec::new();
        gen_key_pair_sigs(8, &mut priv_pub_keys, &mut signatures, &vote_msg);

        let sigs_pubkeys = signatures
            .iter()
            .zip(priv_pub_keys.iter())
            .map(|(sig, key_pair)| (sig.clone(), key_pair.1.clone()))
            .collect::<Vec<_>>();

        b.iter(|| {
            let _ = BlsSignature::combine(sigs_pubkeys.clone());
        })
    });
    // MacOS M1 Pro 16GB: 727.03 µs
    c.bench_function("16 aggregated sig", |b| {
        let vote_msg =
            HashValue::try_from(Hasher::digest(Bytes::from(rlp::encode(&mock_vote()))).as_bytes())
                .unwrap();

        let mut priv_pub_keys = Vec::new();
        let mut signatures = Vec::new();
        gen_key_pair_sigs(16, &mut priv_pub_keys, &mut signatures, &vote_msg);

        let sigs_pubkeys = signatures
            .iter()
            .zip(priv_pub_keys.iter())
            .map(|(sig, key_pair)| (sig.clone(), key_pair.1.clone()))
            .collect::<Vec<_>>();

        b.iter(|| {
            let _ = BlsSignature::combine(sigs_pubkeys.clone());
        })
    });
    // MacOS M1 Pro 16GB: 1.45 ms
    c.bench_function("32 aggregated sig", |b| {
        let vote_msg =
            HashValue::try_from(Hasher::digest(Bytes::from(rlp::encode(&mock_vote()))).as_bytes())
                .unwrap();

        let mut priv_pub_keys = Vec::new();
        let mut signatures = Vec::new();
        gen_key_pair_sigs(32, &mut priv_pub_keys, &mut signatures, &vote_msg);

        let sigs_pubkeys = signatures
            .iter()
            .zip(priv_pub_keys.iter())
            .map(|(sig, key_pair)| (sig.clone(), key_pair.1.clone()))
            .collect::<Vec<_>>();

        b.iter(|| {
            let _ = BlsSignature::combine(sigs_pubkeys.clone());
        })
    });
    // MacOS M1 Pro 16GB: 2.90 ms
    c.bench_function("64 aggregated sig", |b| {
        let vote_msg =
            HashValue::try_from(Hasher::digest(Bytes::from(rlp::encode(&mock_vote()))).as_bytes())
                .unwrap();

        let mut priv_pub_keys = Vec::new();
        let mut signatures = Vec::new();
        gen_key_pair_sigs(64, &mut priv_pub_keys, &mut signatures, &vote_msg);

        let sigs_pubkeys = signatures
            .iter()
            .zip(priv_pub_keys.iter())
            .map(|(sig, key_pair)| (sig.clone(), key_pair.1.clone()))
            .collect::<Vec<_>>();

        b.iter(|| {
            let _ = BlsSignature::combine(sigs_pubkeys.clone());
        })
    });
}

fn criterion_aggregated_sig_verify(c: &mut Criterion) {
    // MacOS M1 Pro 16GB: 804.36 µs
    c.bench_function("4 aggregated sig verify", |b| {
        let vote_msg =
            HashValue::try_from(Hasher::digest(Bytes::from(rlp::encode(&mock_vote()))).as_bytes())
                .unwrap();

        let mut priv_pub_keys = Vec::new();
        let mut signatures = Vec::new();
        gen_key_pair_sigs(4, &mut priv_pub_keys, &mut signatures, &vote_msg);

        let sigs_pubkeys = signatures
            .iter()
            .zip(priv_pub_keys.iter())
            .map(|(sig, key_pair)| (sig.clone(), key_pair.1.clone()))
            .collect::<Vec<_>>();
        let aggragated_sig = BlsSignature::combine(sigs_pubkeys).unwrap();
        let aggregated_key = BlsPublicKey::aggregate(
            priv_pub_keys
                .iter()
                .map(|key_pair| key_pair.1.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();

        b.iter(|| {
            aggragated_sig
                .clone()
                .verify(&vote_msg, &aggregated_key, &String::new())
                .unwrap();
        })
    });
    c.bench_function("8 aggregated sig verify", |b| {
        // MacOS M1 Pro 16GB: 803.95 µs
        let vote_msg =
            HashValue::try_from(Hasher::digest(Bytes::from(rlp::encode(&mock_vote()))).as_bytes())
                .unwrap();

        let mut priv_pub_keys = Vec::new();
        let mut signatures = Vec::new();
        gen_key_pair_sigs(8, &mut priv_pub_keys, &mut signatures, &vote_msg);

        let sigs_pubkeys = signatures
            .iter()
            .zip(priv_pub_keys.iter())
            .map(|(sig, key_pair)| (sig.clone(), key_pair.1.clone()))
            .collect::<Vec<_>>();
        let aggragated_sig = BlsSignature::combine(sigs_pubkeys).unwrap();
        let aggregated_key = BlsPublicKey::aggregate(
            priv_pub_keys
                .iter()
                .map(|key_pair| key_pair.1.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();

        b.iter(|| {
            aggragated_sig
                .clone()
                .verify(&vote_msg, &aggregated_key, &String::new())
                .unwrap();
        })
    });
    // MacOS M1 Pro 16GB: 811.12 µs
    c.bench_function("16 aggregated sig verify", |b| {
        let vote_msg =
            HashValue::try_from(Hasher::digest(Bytes::from(rlp::encode(&mock_vote()))).as_bytes())
                .unwrap();

        let mut priv_pub_keys = Vec::new();
        let mut signatures = Vec::new();
        gen_key_pair_sigs(16, &mut priv_pub_keys, &mut signatures, &vote_msg);

        let sigs_pubkeys = signatures
            .iter()
            .zip(priv_pub_keys.iter())
            .map(|(sig, key_pair)| (sig.clone(), key_pair.1.clone()))
            .collect::<Vec<_>>();
        let aggragated_sig = BlsSignature::combine(sigs_pubkeys).unwrap();
        let aggregated_key = BlsPublicKey::aggregate(
            priv_pub_keys
                .iter()
                .map(|key_pair| key_pair.1.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();

        b.iter(|| {
            aggragated_sig
                .clone()
                .verify(&vote_msg, &aggregated_key, &String::new())
                .unwrap();
        })
    });
    // MacOS M1 Pro 16GB: 800.65 µs
    c.bench_function("32 aggregated sig verify", |b| {
        let vote_msg =
            HashValue::try_from(Hasher::digest(Bytes::from(rlp::encode(&mock_vote()))).as_bytes())
                .unwrap();

        let mut priv_pub_keys = Vec::new();
        let mut signatures = Vec::new();
        gen_key_pair_sigs(32, &mut priv_pub_keys, &mut signatures, &vote_msg);

        let sigs_pubkeys = signatures
            .iter()
            .zip(priv_pub_keys.iter())
            .map(|(sig, key_pair)| (sig.clone(), key_pair.1.clone()))
            .collect::<Vec<_>>();
        let aggragated_sig = BlsSignature::combine(sigs_pubkeys).unwrap();
        let aggregated_key = BlsPublicKey::aggregate(
            priv_pub_keys
                .iter()
                .map(|key_pair| key_pair.1.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();

        b.iter(|| {
            aggragated_sig
                .clone()
                .verify(&vote_msg, &aggregated_key, &String::new())
                .unwrap();
        })
    });
    // MacOS M1 Pro 16GB: 853.88 µs
    c.bench_function("64 aggregated sig verify", |b| {
        let vote_msg =
            HashValue::try_from(Hasher::digest(Bytes::from(rlp::encode(&mock_vote()))).as_bytes())
                .unwrap();

        let mut priv_pub_keys = Vec::new();
        let mut signatures = Vec::new();
        gen_key_pair_sigs(64, &mut priv_pub_keys, &mut signatures, &vote_msg);

        let sigs_pubkeys = signatures
            .iter()
            .zip(priv_pub_keys.iter())
            .map(|(sig, key_pair)| (sig.clone(), key_pair.1.clone()))
            .collect::<Vec<_>>();
        let aggragated_sig = BlsSignature::combine(sigs_pubkeys).unwrap();
        let aggregated_key = BlsPublicKey::aggregate(
            priv_pub_keys
                .iter()
                .map(|key_pair| key_pair.1.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();

        b.iter(move || {
            aggragated_sig
                .clone()
                .verify(&vote_msg, &aggregated_key, &String::new())
                .unwrap();
        })
    });
}

fn mock_block_hash() -> H256 {
    protocol::types::Hash::random()
}

fn mock_vote() -> Vote {
    Vote {
        height:     0u64,
        round:      0u64,
        vote_type:  VoteType::Prevote,
        block_hash: Bytes::from(mock_block_hash().as_bytes().to_vec()),
    }
}

fn gen_key_pair_sigs(
    size: usize,
    keypairs: &mut Vec<(BlsPrivateKey, BlsPublicKey)>,
    sigs: &mut Vec<BlsSignature>,
    hash: &HashValue,
) {
    for _i in 0..size {
        let bls_priv_key = BlsPrivateKey::generate(&mut rand::rngs::OsRng);
        let bls_pub_key = bls_priv_key.pub_key(&String::new());

        let sig = bls_priv_key.sign_message(hash);
        keypairs.push((bls_priv_key, bls_pub_key));
        sigs.push(sig);
    }
}

criterion_group!(
    benches,
    criterion_aggregated_sig,
    criterion_aggregated_sig_verify
);
criterion_main!(benches);
