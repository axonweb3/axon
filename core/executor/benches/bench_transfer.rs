use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use criterion::{criterion_group, criterion_main, Criterion};
use rand::random;

use common_crypto::{
    Crypto, PrivateKey, Secp256k1Recoverable, Secp256k1RecoverablePrivateKey, Signature,
    ToPublicKey, UncompressedPublicKey,
};
use core_executor::{EVMExecutorAdapter, EvmExecutor, MPTTrie, RocksTrieDB};
use core_storage::{adapter::rocks::RocksAdapter, ImplStorage};
use protocol::types::{
    public_to_address, Account, Address, ExecutorContext, Hash, Public, SignedTransaction,
    Transaction, TransactionAction, UnverifiedTransaction, NIL_DATA, RLP_NULL, U256,
};
use protocol::{codec::ProtocolCodec, traits::Executor};

lazy_static::lazy_static! {
    static ref PRIVITE_KEY: Secp256k1RecoverablePrivateKey
        = Secp256k1RecoverablePrivateKey::try_from(hex::decode("95500289866f83502cc1fb894ef5e2b840ca5f867cc9e84ab32fb8872b5dd36c").unwrap().as_ref()).unwrap();
    static ref DISTRIBUTE_ADDRESS: Address = Address::from_hex("0x35e70c3f5a794a77efc2ec5ba964bffcc7fd2c0a").unwrap();
}

const STATE_PATH: &str = "../../free-space/rocks/state";
const DATA_PATH: &str = "../../free-space/rocks/data";

struct BenchAdapter {
    trie_db: Arc<RocksTrieDB>,
    storage: Arc<ImplStorage<RocksAdapter>>,
}

impl BenchAdapter {
    fn new() -> Self {
        BenchAdapter {
            trie_db: Arc::new(RocksTrieDB::new(STATE_PATH, 1000, 1000).unwrap()),
            storage: Arc::new(ImplStorage::new(Arc::new(
                RocksAdapter::new(DATA_PATH, 1000).unwrap(),
            ))),
        }
    }

    fn init_mpt(&self) -> Hash {
        let mut mpt = MPTTrie::new(Arc::clone(&self.trie_db));
        let distribute_account = Account {
            nonce:        0u64.into(),
            balance:      32000001100000000000u128.into(),
            storage_root: RLP_NULL,
            code_hash:    NIL_DATA,
        };

        mpt.insert(
            DISTRIBUTE_ADDRESS.as_slice(),
            distribute_account.encode().unwrap().as_ref(),
        )
        .unwrap();

        mpt.commit().unwrap()
    }

    fn init_bachend(&self) -> EVMExecutorAdapter<ImplStorage<RocksAdapter>, RocksTrieDB> {
        EVMExecutorAdapter::from_root(
            self.init_mpt(),
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            ExecutorContext {
                block_number:           1u64.into(),
                block_hash:             rand_hash(),
                block_coinbase:         DISTRIBUTE_ADDRESS.0,
                block_timestamp:        time_now().into(),
                chain_id:               U256::one(),
                difficulty:             U256::one(),
                origin:                 DISTRIBUTE_ADDRESS.0,
                gas_price:              85u64.into(),
                block_gas_limit:        100_000_000_000u64.into(),
                block_base_fee_per_gas: Default::default(),
                logs:                   vec![],
            },
        )
        .unwrap()
    }
}

fn rand_hash() -> Hash {
    Hash::from_slice(&(0..32).map(|_| random()).collect::<Vec<_>>())
}

fn time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn mock_transaction(nonce: u64) -> SignedTransaction {
    let tx = Transaction {
        nonce:                    nonce.into(),
        max_priority_fee_per_gas: 1u64.into(),
        gas_price:                85u64.into(),
        gas_limit:                1000000u64.into(),
        value:                    10u64.into(),
        data:                     Default::default(),
        access_list:              vec![],
        action:                   TransactionAction::Call(
            Address::from_hex("0x829BD824B016326A401d083B33D092293333A830")
                .unwrap()
                .0,
        ),
    };

    let mut utx = UnverifiedTransaction {
        unsigned:  tx,
        signature: None,
        chain_id:  0,
        hash:      Default::default(),
    };

    let raw = utx.signature_hash();
    let signature = Secp256k1Recoverable::sign_message(raw.as_bytes(), &PRIVITE_KEY.to_bytes())
        .unwrap()
        .to_bytes();
    utx.signature = Some(signature.into());

    let pub_key = Public::from_slice(&PRIVITE_KEY.pub_key().to_uncompressed_bytes()[1..65]);

    SignedTransaction {
        transaction: utx.hash(),
        sender:      public_to_address(&pub_key),
        public:      Some(pub_key),
    }
}

fn mock_txs(number: u64) -> Vec<SignedTransaction> {
    (0..number).map(mock_transaction).collect()
}

fn criterion_100_txs(c: &mut Criterion) {
    let txs = mock_txs(100);
    let executor = EvmExecutor::default();
    let mut backend = BenchAdapter::new().init_bachend();

    c.bench_function("transfer 100", |b| {
        b.iter(|| executor.exec(&mut backend, txs.clone()))
    });
}

fn criterion_1000_txs(c: &mut Criterion) {
    let txs = mock_txs(1000);
    let executor = EvmExecutor::default();
    let mut backend = BenchAdapter::new().init_bachend();

    c.bench_function("transfer 1000", |b| {
        b.iter(|| executor.exec(&mut backend, txs.clone()))
    });
}

fn criterion_10000_txs(c: &mut Criterion) {
    let txs = mock_txs(10000);
    let executor = EvmExecutor::default();
    let mut backend = BenchAdapter::new().init_bachend();

    c.bench_function("transfer 10000", |b| {
        b.iter(|| executor.exec(&mut backend, txs.clone()))
    });
}

criterion_group!(
    benches,
    criterion_100_txs,
    criterion_1000_txs,
    criterion_10000_txs,
);
criterion_main!(benches);
