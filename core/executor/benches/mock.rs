use std::sync::Arc;

use common_crypto::{
    Crypto, PrivateKey, Secp256k1Recoverable, Secp256k1RecoverablePrivateKey, Signature,
    ToPublicKey, UncompressedPublicKey,
};
use core_executor::RocksTrieDB;
use core_storage::adapter::rocks::RocksAdapter;
use core_storage::ImplStorage;
use protocol::{
    codec::hex_decode,
    types::{
        public_to_address, Account, Address, Bytes, Eip1559Transaction, ExecutorContext, Public,
        SignedTransaction, TransactionAction, UnsignedTransaction, UnverifiedTransaction, H160,
        H512, NIL_DATA, RLP_NULL, U256,
    },
};

lazy_static::lazy_static! {
    static ref PRIVATE_KEY: Secp256k1RecoverablePrivateKey
        = Secp256k1RecoverablePrivateKey::try_from(hex_decode("95500289866f83502cc1fb894ef5e2b840ca5f867cc9e84ab32fb8872b5dd36c").unwrap().as_ref()).unwrap();
    static ref DISTRIBUTE_ADDRESS: Address = Address::from_hex("0x35e70c3f5a794a77efc2ec5ba964bffcc7fd2c0a").unwrap();
}

const STATE_PATH: &str = "../../free-space/rocks/state";
const DATA_PATH: &str = "../../free-space/rocks/data";

pub fn new_rocks_trie_db() -> RocksTrieDB {
    RocksTrieDB::new(STATE_PATH, Default::default(), 1000).unwrap()
}

pub fn new_storage() -> ImplStorage<RocksAdapter> {
    ImplStorage::new(
        Arc::new(RocksAdapter::new(DATA_PATH, Default::default()).unwrap()),
        100,
    )
}

pub fn init_account() -> (Account, Address) {
    let account = Account {
        nonce:        0u64.into(),
        balance:      111_320_000_011_u64.into(),
        storage_root: RLP_NULL,
        code_hash:    NIL_DATA,
    };
    (account, DISTRIBUTE_ADDRESS.clone())
}

pub fn mock_executor_context() -> ExecutorContext {
    ExecutorContext {
        block_number:           1u64.into(),
        block_coinbase:         DISTRIBUTE_ADDRESS.0,
        block_timestamp:        time_now().into(),
        chain_id:               U256::one(),
        origin:                 DISTRIBUTE_ADDRESS.0,
        gas_price:              85u64.into(),
        block_gas_limit:        100_000_000_000u64.into(),
        block_base_fee_per_gas: Default::default(),
        logs:                   vec![],
    }
}

// Generate the private and public key and transfer.
pub fn mock_transactions(n: usize) -> Vec<SignedTransaction> {
    let key_pairs = gen_key_pairs(n);
    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        let (sender_addr, sender_pub_key, sender_priv_key) = if i == 0 {
            let pub_key = Public::from_slice(&PRIVATE_KEY.pub_key().to_uncompressed_bytes()[1..65]);
            (public_to_address(&pub_key), pub_key, PRIVATE_KEY.to_bytes())
        } else {
            let idx = n - 1;
            let key_pair = key_pairs.get(idx).unwrap();
            (
                key_pair.addr,
                key_pair.public_key,
                key_pair.private_key.clone(),
            )
        };
        let raw_tx = Eip1559Transaction {
            nonce:                    U256::zero(),
            max_priority_fee_per_gas: 1u64.into(),
            gas_price:                1u64.into(),
            gas_limit:                10_000_000u64.into(),
            action:                   TransactionAction::Call(key_pairs.get(i).unwrap().addr),
            value:                    (1_000_000_000_u64 - 1000_u64 * i as u64).into(),
            data:                     Bytes::default(),
            access_list:              vec![],
        };
        let tx = {
            let mut utx = UnverifiedTransaction {
                unsigned:  UnsignedTransaction::Eip1559(raw_tx),
                signature: None,
                chain_id:  Some(0u64),
                hash:      Default::default(),
            };
            let hash = utx.signature_hash(true);
            let signature = Secp256k1Recoverable::sign_message(hash.as_bytes(), &sender_priv_key)
                .unwrap()
                .to_bytes();
            utx.signature = Some(signature.into());
            SignedTransaction {
                transaction: utx.calc_hash(),
                sender:      sender_addr,
                public:      Some(sender_pub_key),
            }
        };
        result.push(tx);
    }
    result
}

struct KeyPair {
    pub private_key: Bytes,
    pub addr:        H160,
    pub public_key:  H512,
}

impl From<Secp256k1RecoverablePrivateKey> for KeyPair {
    fn from(private: Secp256k1RecoverablePrivateKey) -> Self {
        let original_pub_key = private.pub_key();
        let pub_bytes = &original_pub_key.to_uncompressed_bytes()[1..65];
        let pub_key = Public::from_slice(pub_bytes);
        let pub_addr = public_to_address(&pub_key);
        KeyPair {
            private_key: private.to_bytes(),
            addr:        pub_addr,
            public_key:  pub_key,
        }
    }
}

fn gen_key_pairs(n: usize) -> Vec<KeyPair> {
    use protocol::rand::rngs::OsRng;

    let mut result = Vec::with_capacity(n);
    for _ in 0..n {
        let private_key = Secp256k1RecoverablePrivateKey::generate(&mut OsRng);
        result.push(KeyPair::from(private_key));
    }

    result
}

#[inline]
fn time_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
