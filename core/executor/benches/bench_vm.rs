mod mock;
mod revm_adapter;

use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};

use core_executor::{AxonExecutor, AxonExecutorAdapter, MPTTrie};
use protocol::{
    codec::ProtocolCodec,
    traits::{Executor, Storage},
    types::{Account, Address, ExecutorContext},
};

use mock::{
    init_account, mock_executor_context, mock_transactions, new_rocks_trie_db, new_storage,
};
use revm_adapter::{revm_exec, RevmAdapter};

trait BackendInit<S: Storage + 'static, DB: cita_trie::DB + 'static> {
    fn init(
        storage: S,
        db: DB,
        exec_ctx: ExecutorContext,
        init_account: Account,
        addr: Address,
    ) -> Self;
}

impl<S, DB> BackendInit<S, DB> for RevmAdapter<S, DB>
where
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    fn init(
        storage: S,
        db: DB,
        exec_ctx: ExecutorContext,
        init_account: Account,
        addr: Address,
    ) -> Self {
        let mut revm_adapter = RevmAdapter::new(storage, db, exec_ctx);
        revm_adapter.init_mpt(init_account, addr);
        revm_adapter
    }
}

impl<S, DB> BackendInit<S, DB> for AxonExecutorAdapter<S, DB>
where
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    fn init(
        storage: S,
        db: DB,
        exec_ctx: ExecutorContext,
        init_account: Account,
        addr: Address,
    ) -> Self {
        let db = Arc::new(db);
        let mut mpt = MPTTrie::new(Arc::clone(&db));

        mpt.insert(addr.as_slice(), init_account.encode().unwrap().as_ref())
            .unwrap();

        let state_root = mpt.commit().unwrap();
        AxonExecutorAdapter::from_root(state_root, db, Arc::new(storage), exec_ctx).unwrap()
    }
}

fn criterion_10000_txs(c: &mut Criterion) {
    let txs = mock_transactions(10000);
    // MacOS M1 Pro, 16GB: time: 73.397ms
    c.bench_function("revm 10000 tx", |b| {
        let storage = new_storage();
        let db = new_rocks_trie_db();
        let exec_ctx = mock_executor_context();
        let (account, addr) = init_account();
        let revm_adapter = RevmAdapter::init(storage, db, exec_ctx, account, addr);
        let mut evm = revm::EVM::new();
        evm.database(revm_adapter);
        b.iter(|| {
            revm_exec(&mut evm, txs.clone());
        });
    });
    // MacOS M1 Pro, 16GB: time: 56.627ms
    c.bench_function("evm 10000 tx", |b| {
        let storage = new_storage();
        let db = new_rocks_trie_db();
        let exec_ctx = mock_executor_context();
        let (account, addr) = init_account();
        let mut axon_adapter = AxonExecutorAdapter::init(storage, db, exec_ctx, account, addr);
        let executor = AxonExecutor::default();
        b.iter(|| {
            executor.exec(&mut axon_adapter, &txs);
        })
    });
}

criterion_group!(benches, criterion_10000_txs);
criterion_main!(benches);
