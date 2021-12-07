extern crate test;

mod mempool;

use std::convert::{From, TryFrom};
use std::sync::Arc;

use chashmap::CHashMap;
use futures::executor;
use rand::random;
use rand::rngs::OsRng;

use common_crypto::{
    Crypto, PrivateKey, PublicKey, Secp256k1, Secp256k1PrivateKey, Secp256k1PublicKey,
    Secp256k1Signature, Signature, ToPublicKey,
};
use protocol::codec::ProtocolCodec;
use protocol::traits::{Context, MemPool, MemPoolAdapter, MixedTxHashes};
use protocol::types::{
    Address, Bytes, Hash, Hasher, Public, SignatureComponents, SignedTransaction, Transaction,
    TransactionAction, UnverifiedTransaction, H256, U256,
};
use protocol::{async_trait, tokio, ProtocolResult};

use crate::{check_dup_order_hashes, HashMemPool, MemPoolError};

const CYCLE_LIMIT: u64 = 1_000_000;
const TX_NUM_LIMIT: u64 = 10_000;
const CURRENT_HEIGHT: u64 = 999;
const POOL_SIZE: usize = 100_000;
const MAX_TX_SIZE: u64 = 1024; // 1KB
const TIMEOUT: u64 = 1000;
const TIMEOUT_GAP: u64 = 100;

pub struct HashMemPoolAdapter {
    network_txs: CHashMap<Hash, SignedTransaction>,
}

impl HashMemPoolAdapter {
    fn new() -> HashMemPoolAdapter {
        HashMemPoolAdapter {
            network_txs: CHashMap::new(),
        }
    }
}

#[async_trait]
impl MemPoolAdapter for HashMemPoolAdapter {
    async fn pull_txs(
        &self,
        _ctx: Context,
        _height: Option<u64>,
        tx_hashes: Vec<Hash>,
    ) -> ProtocolResult<Vec<SignedTransaction>> {
        let mut vec = Vec::new();
        for hash in tx_hashes {
            if let Some(tx) = self.network_txs.get(&hash) {
                vec.push(tx.clone());
            }
        }
        Ok(vec)
    }

    async fn broadcast_tx(&self, _ctx: Context, tx: SignedTransaction) -> ProtocolResult<()> {
        self.network_txs.insert(tx.transaction.hash, tx);
        Ok(())
    }

    async fn check_authorization(
        &self,
        _ctx: Context,
        tx: Box<SignedTransaction>,
    ) -> ProtocolResult<()> {
        check_hash(&tx.clone()).await?;
        check_sig(&tx)
    }

    async fn check_transaction(
        &self,
        _ctx: Context,
        _tx: &SignedTransaction,
    ) -> ProtocolResult<()> {
        Ok(())
    }

    async fn check_storage_exist(&self, _ctx: Context, _tx_hash: &Hash) -> ProtocolResult<()> {
        Ok(())
    }

    async fn get_latest_height(&self, _ctx: Context) -> ProtocolResult<u64> {
        Ok(CURRENT_HEIGHT)
    }

    async fn get_transactions_from_storage(
        &self,
        _ctx: Context,
        _height: Option<u64>,
        _tx_hashes: &[Hash],
    ) -> ProtocolResult<Vec<Option<SignedTransaction>>> {
        Ok(vec![])
    }

    fn report_good(&self, _ctx: Context) {}
}

pub fn default_mock_txs(size: usize) -> Vec<SignedTransaction> {
    mock_txs(size, 0, TIMEOUT)
}

fn mock_txs(valid_size: usize, invalid_size: usize, timeout: u64) -> Vec<SignedTransaction> {
    let mut vec = Vec::new();
    let priv_key = Secp256k1PrivateKey::generate(&mut OsRng);
    let pub_key = priv_key.pub_key();
    for i in 0..valid_size + invalid_size {
        vec.push(mock_signed_tx());
    }
    vec
}

fn default_mempool_sync() -> HashMemPool<HashMemPoolAdapter> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(default_mempool())
}

async fn default_mempool() -> HashMemPool<HashMemPoolAdapter> {
    new_mempool(POOL_SIZE, TIMEOUT_GAP, CYCLE_LIMIT, MAX_TX_SIZE).await
}

async fn new_mempool(
    pool_size: usize,
    timeout_gap: u64,
    cycles_limit: u64,
    max_tx_size: u64,
) -> HashMemPool<HashMemPoolAdapter> {
    let adapter = HashMemPoolAdapter::new();
    let mempool = HashMemPool::new(pool_size, adapter, vec![]).await;
    mempool
}

async fn check_hash(tx: &SignedTransaction) -> ProtocolResult<()> {
    let raw = tx.transaction.unsigned.clone();
    let raw_bytes = raw.encode()?;
    let tx_hash = Hasher::digest(raw_bytes);
    if tx_hash != tx.transaction.hash {
        return Err(MemPoolError::CheckHash {
            expect: tx.transaction.hash,
            actual: tx_hash,
        }
        .into());
    }
    Ok(())
}

fn check_sig(tx: &SignedTransaction) -> ProtocolResult<()> {
    Ok(())
}

async fn concurrent_check_sig(txs: Vec<SignedTransaction>) {
    let futs = txs
        .into_iter()
        .map(|tx| tokio::task::spawn_blocking(move || check_sig(&tx).unwrap()))
        .collect::<Vec<_>>();

    futures::future::try_join_all(futs).await.unwrap();
}

async fn concurrent_insert(
    txs: Vec<SignedTransaction>,
    mempool: Arc<HashMemPool<HashMemPoolAdapter>>,
) {
    let futs = txs
        .into_iter()
        .map(|tx| {
            let mempool = Arc::clone(&mempool);
            tokio::spawn(async { exec_insert(tx, mempool).await })
        })
        .collect::<Vec<_>>();

    futures::future::try_join_all(futs).await.unwrap();
}

async fn concurrent_broadcast(
    txs: Vec<SignedTransaction>,
    mempool: Arc<HashMemPool<HashMemPoolAdapter>>,
) {
    let futs = txs
        .into_iter()
        .map(|tx| {
            let mempool = Arc::clone(&mempool);
            tokio::spawn(async move {
                mempool
                    .get_adapter()
                    .broadcast_tx(Context::new(), tx)
                    .await
                    .unwrap()
            })
        })
        .collect::<Vec<_>>();

    futures::future::try_join_all(futs).await.unwrap();
}

async fn exec_insert(signed_tx: SignedTransaction, mempool: Arc<HashMemPool<HashMemPoolAdapter>>) {
    let _ = mempool.insert(Context::new(), signed_tx).await.is_ok();
}

async fn exec_flush(remove_hashes: Vec<Hash>, mempool: Arc<HashMemPool<HashMemPoolAdapter>>) {
    mempool.flush(Context::new(), &remove_hashes).await.unwrap()
}

async fn exec_package(
    mempool: Arc<HashMemPool<HashMemPoolAdapter>>,
    cycle_limit: u64,
    tx_num_limit: u64,
) -> MixedTxHashes {
    mempool
        .package(Context::new(), cycle_limit, tx_num_limit)
        .await
        .unwrap()
}

async fn exec_ensure_order_txs(
    require_hashes: Vec<Hash>,
    mempool: Arc<HashMemPool<HashMemPoolAdapter>>,
) {
    mempool
        .ensure_order_txs(Context::new(), None, &require_hashes)
        .await
        .unwrap();
}

async fn exec_sync_propose_txs(
    require_hashes: Vec<Hash>,
    mempool: Arc<HashMemPool<HashMemPoolAdapter>>,
) {
    mempool
        .sync_propose_txs(Context::new(), require_hashes)
        .await
        .unwrap();
}

async fn exec_get_full_txs(
    require_hashes: Vec<Hash>,
    mempool: Arc<HashMemPool<HashMemPoolAdapter>>,
) -> Vec<SignedTransaction> {
    mempool
        .get_full_txs(Context::new(), None, &require_hashes)
        .await
        .unwrap()
}

fn mock_transaction() -> Transaction {
    Transaction {
        chain_id:                 random::<u64>(),
        nonce:                    U256::one(),
        gas_limit:                U256::one(),
        max_priority_fee_per_gas: U256::one(),
        max_fee_per_gas:          U256::one(),
        action:                   TransactionAction::Create,
        value:                    U256::one(),
        input:                    rand_bytes(32).to_vec(),
        access_list:              vec![],
        odd_y_parity:             true,
        r:                        H256::default(),
        s:                        H256::default(),
    }
}

fn mock_sig_component() -> SignatureComponents {
    SignatureComponents {
        standard_v: random::<u8>(),
        r:          U256::one(),
        s:          U256::one(),
    }
}

fn mock_unverfied_tx(chain_id: Option<u64>) -> UnverifiedTransaction {
    let tx = mock_transaction();
    UnverifiedTransaction {
        unsigned: tx.clone(),
        chain_id,
        hash: Hasher::digest(tx.encode().unwrap()),
        signature: mock_sig_component(),
    }
}

fn mock_signed_tx() -> SignedTransaction {
    SignedTransaction {
        transaction: mock_unverfied_tx(Some(random::<u64>())),
        sender:      Address::default(),
        public:      Some(Public::default()),
    }
}

fn _get_random_bytes(len: usize) -> Vec<u8> {
    (0..len).map(|_| random::<u8>()).collect()
}

fn check_order_consistant(mixed_tx_hashes: &MixedTxHashes, txs: &[SignedTransaction]) -> bool {
    mixed_tx_hashes
        .order_tx_hashes
        .iter()
        .enumerate()
        .any(|(i, hash)| hash == &txs.get(i).unwrap().transaction.hash)
}

fn rand_bytes(len: usize) -> Bytes {
    Bytes::from((0..len).map(|_| random::<u8>()).collect::<Vec<_>>())
}
