use criterion::{criterion_group, criterion_main, Criterion};

use common_merkle::*;
use protocol::types::Hash;

fn mock_hash() -> Hash {
    Hash::random()
}

fn rand_hashes(size: usize) -> Vec<Hash> {
    (0..size).map(|_| mock_hash()).collect::<Vec<_>>()
}

fn criterion_merkle(c: &mut Criterion) {
    // MacOS M1 Pro 16G: 345.85µs
    c.bench_function("merkle 1000 hashes", |b| {
        let case = rand_hashes(1000);
        b.iter(|| {
            let _ = Merkle::from_hashes(case.clone());
        });
    });
    // MacOS M1 Pro 16G: 697.92µs
    c.bench_function("merkle 2000 hashes", |b| {
        let case = rand_hashes(2000);
        b.iter(|| {
            let _ = Merkle::from_hashes(case.clone());
        });
    });
    // MacOS M1 Pro 16G: 1.41ms
    c.bench_function("merkle 4000 hashes", |b| {
        let case = rand_hashes(4000);
        b.iter(|| {
            let _ = Merkle::from_hashes(case.clone());
        });
    });
    // MacOS M1 Pro 16G: 2.84ms
    c.bench_function("merkle 8000 hashes", |b| {
        let case = rand_hashes(8000);
        b.iter(|| {
            let _ = Merkle::from_hashes(case.clone());
        });
    });
    // MacOS M1 Pro 16G: 5.65ms
    c.bench_function("merkle 16000 hashes", |b| {
        let case = rand_hashes(16000);
        b.iter(|| {
            let _ = Merkle::from_hashes(case.clone());
        });
    });
}

criterion_group!(benches, criterion_merkle);
criterion_main!(benches);
