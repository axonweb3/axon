use std::collections::HashSet;
use std::sync::Arc;

use protocol::types::Hasher;

use super::*;

macro_rules! insert {
    (normal($pool_size: expr, $input: expr, $output: expr)) => {
        insert!(inner($pool_size, 1, $input, 0, $output));
    };
    (repeat($repeat: expr, $input: expr, $output: expr)) => {
        insert!(inner($input * 10, $repeat, $input, 0, $output));
    };
    (invalid($valid: expr, $invalid: expr, $output: expr)) => {
        insert!(inner($valid * 10, 1, $valid, $invalid, $output));
    };
    (inner($pool_size: expr, $repeat: expr, $valid: expr, $invalid: expr, $output: expr)) => {
        let mempool =
            Arc::new(new_mempool($pool_size, TIMEOUT_GAP, CYCLE_LIMIT, MAX_TX_SIZE).await);
        let txs = mock_txs($valid, $invalid, TIMEOUT);
        for _ in 0..$repeat {
            concurrent_insert(txs.clone(), Arc::clone(&mempool)).await;
        }
        assert_eq!(mempool.get_tx_cache().len(), $output);
    };
}

#[test]
fn test_dup_order_hashes() {
    let hashes = vec![
        Hasher::digest(Bytes::from("test1")),
        Hasher::digest(Bytes::from("test2")),
        Hasher::digest(Bytes::from("test3")),
        Hasher::digest(Bytes::from("test4")),
        Hasher::digest(Bytes::from("test2")),
    ];
    assert!(check_dup_order_hashes(&hashes).is_err());

    let hashes = vec![
        Hasher::digest(Bytes::from("test1")),
        Hasher::digest(Bytes::from("test2")),
        Hasher::digest(Bytes::from("test3")),
        Hasher::digest(Bytes::from("test4")),
    ];
    assert!(check_dup_order_hashes(&hashes).is_ok());
}

#[tokio::test]
async fn test_insert() {
    // 1. insertion under pool size.
    insert!(normal(100, 100, 100));

    // 2. invalid insertion
    insert!(invalid(80, 10, 80));
}

macro_rules! package {
    (normal($tx_num_limit: expr, $insert: expr, $expect_order: expr, $expect_propose: expr)) => {
        package!(inner(
            $tx_num_limit,
            TIMEOUT_GAP,
            TIMEOUT,
            $insert,
            $expect_order,
            $expect_propose
        ));
    };
    (timeout($timeout_gap: expr, $timeout: expr, $insert: expr, $expect: expr)) => {
        package!(inner($insert, $timeout_gap, $timeout, $insert, $expect, 0));
    };
    (inner($tx_num_limit: expr, $timeout_gap: expr, $timeout: expr, $insert: expr, $expect_order: expr, $expect_propose: expr)) => {
        let mempool =
            &Arc::new(new_mempool($insert * 10, $timeout_gap, CYCLE_LIMIT, MAX_TX_SIZE).await);
        let txs = mock_txs($insert, 0, $timeout);
        concurrent_insert(txs.clone(), Arc::clone(mempool)).await;
        protocol::tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let tx_hashes = exec_package(Arc::clone(mempool), CYCLE_LIMIT.into(), $tx_num_limit).await;
        assert_eq!(tx_hashes.hashes.len(), $expect_order);
    };
}

#[tokio::test]
async fn test_package() {
    // 1. pool_size <= tx_num_limit
    package!(normal(100, 50, 50, 0));
    package!(normal(100, 100, 100, 0));

    // 2. tx_num_limit < pool_size <= 2 * tx_num_limit
    package!(normal(100, 101, 100, 0));
    package!(normal(100, 200, 100, 0));

    // 3. 2 * tx_num_limit < pool_size
    package!(normal(100, 201, 100, 0));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_package_multi_types() {
    let mempool = Arc::new(new_mempool(1024, 0, 0, 0).await);

    // insert txs
    let evm_txs = default_mock_txs(1024);
    let sys_txs = mock_sys_txs(5);
    let mut txs = sys_txs.clone();
    txs.extend_from_slice(&evm_txs);
    concurrent_insert(txs.clone(), Arc::clone(&mempool)).await;
    assert_eq!(mempool.get_tx_cache().system_script_queue_len(), 5);

    let package_txs = mempool
        .package(Context::new(), 1000000000u64.into(), 10000)
        .await
        .unwrap();
    assert_eq!(
        sys_txs
            .iter()
            .map(|x| &x.transaction.hash)
            .collect::<HashSet<_>>(),
        package_txs.hashes.iter().take(5).collect::<HashSet<_>>()
    );
    assert_eq!(package_txs.hashes.len(), 1024);

    exec_flush(package_txs.hashes, Arc::clone(&mempool)).await;
    assert_eq!(mempool.get_tx_cache().system_script_queue_len(), 0);
    assert_eq!(mempool.len(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_flush() {
    let mempool = Arc::new(default_mempool().await);

    // insert txs
    let txs = default_mock_txs(555);
    concurrent_insert(txs.clone(), Arc::clone(&mempool)).await;
    assert_eq!(mempool.get_tx_cache().len(), 555);

    // flush exist txs
    let (remove_txs, _) = txs.split_at(123);
    let remove_hashes: Vec<Hash> = remove_txs.iter().map(|tx| tx.transaction.hash).collect();
    exec_flush(remove_hashes, Arc::clone(&mempool)).await;
    assert_eq!(mempool.len(), 432);
    exec_package(Arc::clone(&mempool), CYCLE_LIMIT.into(), TX_NUM_LIMIT).await;
    assert_eq!(mempool.len(), 432);

    // flush absent txs
    let txs = default_mock_txs(222);
    let remove_hashes: Vec<Hash> = txs.iter().map(|tx| tx.transaction.hash).collect();
    exec_flush(remove_hashes, Arc::clone(&mempool)).await;
    assert_eq!(mempool.get_tx_cache().len(), 432);
    assert_eq!(mempool.get_tx_cache().real_queue_len(), 432);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_flush_with_concurrent_insert() {
    let mempool = Arc::new(new_mempool(1024, 0, 0, 0).await);

    // insert txs
    let txs = default_mock_txs(1024);
    concurrent_insert(txs.clone(), Arc::clone(&mempool)).await;
    assert_eq!(mempool.get_tx_cache().len(), 1024);

    let (remove_txs, retain_txs) = txs.split_at(100);
    let remove_hashes: Vec<Hash> = remove_txs.iter().map(|tx| tx.transaction.hash).collect();

    // flush with concurrent insert will never panic
    let txs_two = default_mock_txs(300);
    let j = tokio::spawn(concurrent_insert(txs_two.clone(), Arc::clone(&mempool)));
    exec_flush(remove_hashes, Arc::clone(&mempool)).await;
    j.await.unwrap();

    // all retain tx will on mempool
    let cache_pool = mempool.get_tx_cache();
    for tx in retain_txs {
        assert!(cache_pool.contains(&tx.transaction.hash))
    }

    if cache_pool.len() > 1024 - 100 {
        let mut new_tx = 0;
        for tx in txs_two {
            if cache_pool.contains(&tx.transaction.hash) {
                new_tx += 1;
            }
        }

        assert_eq!(new_tx, cache_pool.len() - (1024 - 100))
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_repeat_insertion_with_timout_gap() {
    let mempool = Arc::new(new_mempool(1024, 0, 0, 0).await);

    let priv_key = Secp256k1RecoverablePrivateKey::generate(&mut OsRng);
    let pub_key = priv_key.pub_key();
    let txs: Vec<SignedTransaction> = (0..3)
        .map(|i| mock_signed_tx(&priv_key, &pub_key, 0, i as u64, true))
        .collect();

    let pool = mempool.get_tx_cache();

    pool.insert(txs[0].clone(), false, 0.into()).unwrap();
    pool.insert(txs[1].clone(), false, 1.into()).unwrap();
    pool.insert(txs[2].clone(), false, 2.into()).unwrap();
    // repeat insertion
    pool.insert(txs[2].clone(), false, 2.into()).unwrap();

    pool.timeout_gap
        .lock()
        .entry(0)
        .or_default()
        .insert(txs[2].transaction.hash);

    pool.flush(&[], 20);

    let list = pool.package(1000.into(), 3);

    assert_eq!(
        list.hashes,
        txs[0..2]
            .iter()
            .map(|tx| tx.transaction.hash)
            .collect::<Vec<_>>()
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn test_nonce_insert() {
    let mempool = Arc::new(new_mempool(1024, 0, 0, 0).await);

    let priv_key = Secp256k1RecoverablePrivateKey::generate(&mut OsRng);
    let pub_key = priv_key.pub_key();
    let txs: Vec<SignedTransaction> = (0..5)
        .map(|i| mock_signed_tx(&priv_key, &pub_key, 0, i as u64, true))
        .collect();

    let replace_tx =
        {
            let mut tx = txs[4].clone();
            match tx.transaction.unsigned {
                UnsignedTransaction::Eip1559(ref mut p) => {
                    p.gas_price = 2.into();
                    p.max_priority_fee_per_gas = 2.into();
                }
                UnsignedTransaction::Eip2930(ref mut p) => p.gas_price = 2.into(),
                UnsignedTransaction::Legacy(ref mut p) => p.gas_price = 2.into(),
            }
            tx.transaction.hash = H256::from_low_u64_le(2);

            tx
        };

    let pool = mempool.get_tx_cache();

    pool.insert(txs[2].clone(), false, 2.into()).unwrap();

    assert_eq!(1, pool.len());
    assert_eq!(0, pool.real_queue_len());

    pool.insert(txs[1].clone(), false, 1.into()).unwrap();

    assert_eq!(2, pool.len());
    assert_eq!(0, pool.real_queue_len());

    pool.insert(txs[0].clone(), false, 0.into()).unwrap();

    assert_eq!(3, pool.len());
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    assert_eq!(3, pool.real_queue_len());

    let list = pool.package(1000.into(), 2);

    assert_eq!(
        list.hashes,
        txs[0..2]
            .iter()
            .map(|tx| tx.transaction.hash)
            .collect::<Vec<_>>()
    );

    pool.flush(&list.hashes, 1);
    assert_eq!(1, pool.real_queue_len());
    // here db nonce = 1, so nonce diff = 1
    pool.insert(txs[3].clone(), false, 1.into()).unwrap();
    assert_eq!(2, pool.len());
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    assert_eq!(2, pool.real_queue_len());

    let list = pool.package(1000.into(), 2);

    assert_eq!(
        list.hashes,
        txs[2..4]
            .iter()
            .map(|tx| tx.transaction.hash)
            .collect::<Vec<_>>()
    );

    pool.flush(&list.hashes, 2);
    assert_eq!(0, pool.real_queue_len());
    // here db nonce = 4, so nonce diff = 1
    pool.insert(txs[4].clone(), false, 0.into()).unwrap();
    assert_eq!(1, pool.len());
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    assert_eq!(1, pool.real_queue_len());
    // replace with high price
    pool.insert(replace_tx.clone(), false, 0.into()).unwrap();
    assert_eq!(2, pool.len());
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    assert_eq!(2, pool.real_queue_len());

    let list = pool.package(1000.into(), 2);
    assert_eq!(list.hashes, vec![replace_tx.transaction.hash]);

    pool.flush(&list.hashes, 3);
    assert_eq!(0, pool.len());
    assert_eq!(0, pool.real_queue_len());
}

macro_rules! ensure_order_txs {
    ($in_pool: expr, $out_pool: expr, $pool_size: expr) => {
        let mempool = &Arc::new(new_mempool($pool_size, 0, 0, 0).await);

        let txs = &default_mock_txs($in_pool + $out_pool);
        let (in_pool_txs, out_pool_txs) = txs.split_at($in_pool);
        concurrent_insert(in_pool_txs.to_vec(), Arc::clone(mempool)).await;
        concurrent_broadcast(out_pool_txs.to_vec(), Arc::clone(mempool)).await;

        let tx_hashes: Vec<Hash> = txs.iter().map(|tx| tx.transaction.hash.clone()).collect();
        exec_ensure_order_txs(tx_hashes.clone(), Arc::clone(mempool)).await;

        let fetch_txs = exec_get_full_txs(tx_hashes, Arc::clone(mempool)).await;
        assert_eq!(fetch_txs.len(), txs.len());
    };
}

#[tokio::test]
async fn test_ensure_order_txs() {
    // all txs are in pool
    ensure_order_txs!(100, 0, 100);
    // 50 txs are not in pool
    ensure_order_txs!(50, 50, 100);
    // all txs are not in pool
    ensure_order_txs!(0, 100, 100);

    // pool size reach limit
    ensure_order_txs!(0, 100, 50);
    ensure_order_txs!(50, 50, 50);
}

#[tokio::test]
async fn bench_sign_with_spawn_list() {
    let adapter = Arc::new(HashMemPoolAdapter::new());
    let txs = default_mock_txs(30000);
    let len = txs.len();
    let now = common_apm::Instant::now();

    let futs = txs
        .into_iter()
        .map(|tx| {
            let adapter = Arc::clone(&adapter);
            tokio::spawn(async move {
                adapter
                    .check_authorization(Context::new(), &tx)
                    .await
                    .unwrap();
            })
        })
        .collect::<Vec<_>>();
    futures::future::try_join_all(futs).await.unwrap();

    println!(
        "bench_sign_with_spawn_list size {:?} cost {:?}",
        len,
        now.elapsed()
    );
}

#[tokio::test]
async fn bench_sign() {
    let adapter = HashMemPoolAdapter::new();
    let txs = default_mock_txs(30000).into_iter().collect::<Vec<_>>();
    let now = common_apm::Instant::now();

    for tx in txs.iter() {
        adapter
            .check_authorization(Context::new(), tx)
            .await
            .unwrap();
    }

    println!("bench_sign size {:?} cost {:?}", txs.len(), now.elapsed());
}
