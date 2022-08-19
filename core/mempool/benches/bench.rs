mod mock;

use protocol::tokio;
use protocol::traits::{Context, MemPool};
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use mock::*;

fn criterion_insert(c: &mut Criterion) {
    // MacOS M1 Pro, 16GB: time: 36.701 µs
    c.bench_function("insert serial 1", |b| {
        let mempool = &Arc::new(default_mempool_sync());
        let txs = default_mock_txs(1);

        b.iter(|| {
            futures::executor::block_on(async {
                for tx in txs.clone().into_iter() {
                    let _ = mempool.insert(Context::new(), tx).await;
                }
            });
        })
    });
    // MacOS M1 Pro, 16GB: time: 369.03 µs
    c.bench_function("insert serial 10", |b| {
        let mempool = &Arc::new(default_mempool_sync());
        let txs = default_mock_txs(10);

        b.iter(|| {
            futures::executor::block_on(async {
                for tx in txs.clone().into_iter() {
                    let _ = mempool.insert(Context::new(), tx).await;
                }
            });
        })
    });
    // MacOS M1 Pro, 16GB: time: 3.74 ms
    c.bench_function("insert serial 100", |b| {
        let mempool = &Arc::new(default_mempool_sync());
        let txs = default_mock_txs(100);

        b.iter(|| {
            futures::executor::block_on(async {
                for tx in txs.clone().into_iter() {
                    let _ = mempool.insert(Context::new(), tx).await;
                }
            });
        })
    });
    // MacOS M1 Pro, 16GB: time: 37.57 ms
    c.bench_function("insert serial 1000", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mempool = &Arc::new(default_mempool_sync());
        let txs = default_mock_txs(1000);

        b.iter(|| {
            runtime.block_on(async {
                for tx in txs.clone().into_iter() {
                    let _ = mempool.insert(Context::new(), tx).await;
                }
            });
        })
    });
}

fn criterion_get_full_txs(c: &mut Criterion) {
    // MacOS M1 Pro, 16GB: time: 1.77 ms
    c.bench_function("get 10000 full txs", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mempool = Arc::new(default_mempool_sync());
        let txs = default_mock_txs(10_000);
        let tx_hashes = txs.iter().map(|tx| tx.transaction.hash).collect::<Vec<_>>();
        runtime.block_on(concurrent_insert(txs, Arc::clone(&mempool)));
        b.iter(|| {
            runtime.block_on(exec_get_full_txs(tx_hashes.clone(), Arc::clone(&mempool)));
        });
    });
    // MacOS M1 Pro, 16GB: time: 3.59 ms
    c.bench_function("get 20000 full txs", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mempool = Arc::new(default_mempool_sync());
        let txs = default_mock_txs(20_000);
        let tx_hashes = txs.iter().map(|tx| tx.transaction.hash).collect::<Vec<_>>();
        runtime.block_on(concurrent_insert(txs, Arc::clone(&mempool)));
        b.iter(|| {
            runtime.block_on(exec_get_full_txs(tx_hashes.clone(), Arc::clone(&mempool)));
        });
    });
    // MacOS M1 Pro, 16GB: time: 10.30 ms
    c.bench_function("get 40000 full txs", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mempool = Arc::new(default_mempool_sync());
        let txs = default_mock_txs(40_000);
        let tx_hashes = txs.iter().map(|tx| tx.transaction.hash).collect::<Vec<_>>();
        runtime.block_on(concurrent_insert(txs, Arc::clone(&mempool)));
        b.iter(|| {
            runtime.block_on(exec_get_full_txs(tx_hashes.clone(), Arc::clone(&mempool)));
        });
    });
    // MacOS M1 Pro, 16GB: time: 23.62 ms
    c.bench_function("get 80000 full txs", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mempool = Arc::new(default_mempool_sync());
        let txs = default_mock_txs(80_000);
        let tx_hashes = txs.iter().map(|tx| tx.transaction.hash).collect::<Vec<_>>();
        runtime.block_on(concurrent_insert(txs, Arc::clone(&mempool)));
        b.iter(|| {
            runtime.block_on(exec_get_full_txs(tx_hashes.clone(), Arc::clone(&mempool)));
        });
    });
}

fn criterion_check_sig_serial(c: &mut Criterion) {
    // MacOS M1 Pro, 16GB: time: 323.55 ns
    c.bench_function("check sig serial 1", |b| {
        let txs = default_mock_txs(1);

        b.iter(|| {
            for tx in txs.iter() {
                let _ = check_sig(tx);
            }
        })
    });
    // MacOS M1 Pro, 16GB: time: 2.57 µs
    c.bench_function("check sig serial 10", |b| {
        let txs = default_mock_txs(10);

        b.iter(|| {
            for tx in txs.iter() {
                let _ = check_sig(tx);
            }
        })
    });
    // MacOS M1 Pro, 16GB: time: 29.42 µs
    c.bench_function("check sig serial 100", |b| {
        let txs = default_mock_txs(100);

        b.iter(|| {
            for tx in txs.iter() {
                let _ = check_sig(tx);
            }
        })
    });
    // MacOS M1 Pro, 16GB: time: 431.62 µs
    c.bench_function("check sig serial 1000", |b| {
        let txs = default_mock_txs(1000);

        b.iter(|| {
            for tx in txs.iter() {
                let _ = check_sig(tx);
            }
        })
    });
}

fn criterion_other(c: &mut Criterion) {
    // MacOS M1 Pro, 16GB: time: 1.12 ms
    c.bench_function("check sig", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let txs = &default_mock_txs(100);

        b.iter(|| {
            runtime.block_on(concurrent_check_sig(txs.clone()));
        });
    });
    // MacOS M1 Pro, 16GB: time: 4.46 ms
    c.bench_function("mock txs", |b| {
        b.iter(|| {
            default_mock_txs(100);
        });
    });
    // MacOS M1 Pro, 16GB: time: 1.03 ms
    c.bench_function("flush", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mempool = &Arc::new(default_mempool_sync());
        let txs = &default_mock_txs(100);
        let remove_hashes: &Vec<protocol::types::Hash> =
            &txs.iter().map(|tx| tx.transaction.hash).collect();
        b.iter(|| {
            runtime.block_on(concurrent_insert(txs.clone(), Arc::clone(mempool)));
            runtime.block_on(exec_flush(remove_hashes.clone(), Arc::clone(mempool)));
            runtime.block_on(exec_package(
                Arc::clone(mempool),
                CYCLE_LIMIT.into(),
                TX_NUM_LIMIT,
            ));
        });
    });
    // MacOS M1 Pro, 16GB: time: 5.80 ms
    c.bench_function("insert", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mempool = &Arc::new(default_mempool_sync());

        b.iter(|| {
            let txs = default_mock_txs(100);
            runtime.block_on(concurrent_insert(txs, Arc::clone(mempool)));
        });
    });
    // MacOS M1 Pro, 16GB: time: 448.19 µs
    c.bench_function("package", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mempool = Arc::new(runtime.block_on(default_mempool()));
        let txs = default_mock_txs(20_000);
        runtime.block_on(concurrent_insert(txs, Arc::clone(&mempool)));
        std::thread::sleep(std::time::Duration::from_secs(1));

        assert_eq!(mempool.get_tx_cache().real_queue_len(), 20_000);

        b.iter(|| {
            runtime.block_on(exec_package(
                Arc::clone(&mempool),
                CYCLE_LIMIT.into(),
                TX_NUM_LIMIT,
            ));
        });
    });
}

criterion_group!(
    benches,
    criterion_check_sig_serial,
    criterion_get_full_txs,
    criterion_insert,
    criterion_other,
);
criterion_main!(benches);
