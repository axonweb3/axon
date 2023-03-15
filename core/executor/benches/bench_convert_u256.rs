use criterion::{criterion_group, criterion_main, Criterion};

use protocol::types::U256;

fn u256_to_u8_slice(index: &U256) -> &[u8] {
    let u64_slice = index.as_ref();
    let result: &[u8] = bytemuck::cast_slice(u64_slice);
    result
}

fn u8_slice_to_u256_2(bytes: &[u8]) -> U256 {
    U256::from_little_endian(bytes)
}

fn u8_slice_to_u256(bytes: &[u8]) -> Option<U256> {
    let u64_slice: &[u64] = bytemuck::cast_slice(bytes);
    let array: &[u64; 4] = u64_slice.try_into().expect("incorrect length");
    Some(U256(array.to_owned()))
}

fn u256_to_u8_slice_2(index: U256) -> Vec<u8> {
    let mut bytes = vec![0u8; 32];
    index.to_little_endian(&mut bytes);
    bytes
}

fn criterion_convert(c: &mut Criterion) {
    let num = U256::from(12345678910099u64);
    // MacOS M1 Pro 16G: 855.08ps
    c.bench_function("convert1", |b| {
        b.iter(|| {
            let slice = u256_to_u8_slice(&num);
            let actual = u8_slice_to_u256(slice).unwrap();
            assert_eq!(num, actual);
        });
    });
    // MacOS M1 Pro 16G: 32.216ns
    c.bench_function("convert2", |b| {
        b.iter(|| {
            let slice = u256_to_u8_slice_2(num);
            let actual = u8_slice_to_u256_2(&slice);
            assert_eq!(num, actual);
        });
    });
}

fn criterion_rand(c: &mut Criterion) {
    const RAND_SEED: u64 = 49999;
    use protocol::rand::rngs::SmallRng;
    use protocol::rand::{Rng, SeedableRng};
    // MacOS M1 Pro 16G: 15.80µs
    c.bench_function("gen rand", |b| {
        b.iter(|| {
            let mut rng = SmallRng::seed_from_u64(RAND_SEED);
            for _ in 0..10000 {
                rng.gen_range(10, 1000000);
            }
        })
    });
}

criterion_group!(benches, criterion_convert, criterion_rand);
criterion_main!(benches);
