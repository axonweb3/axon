mod abi;
mod handle;
mod segment;
mod store;

pub use abi::metadata_abi;
pub use handle::MetadataHandle;
pub use store::MetadataStore;

use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;

use arc_swap::ArcSwap;
use ethers::abi::AbiDecode;
use lru::LruCache;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;

use common_config_parser::types::ConfigRocksDB;
use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{
    Apply, Basic, Hasher, Metadata, SignedTransaction, TxResp, H160, H256, U256,
};

use crate::exec_try;
use crate::system_contract::utils::{revert_resp, succeed_resp};
use crate::system_contract::{image_cell::RocksTrieDB, system_contract_address, SystemContract};

type Epoch = u64;

const METADATA_CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(10) };
const METADATA_DB_CACHE_SIZE: usize = 20;

static METADATA_DB: OnceCell<Arc<RocksTrieDB>> = OnceCell::new();

lazy_static::lazy_static! {
    static ref METADATA_ROOT_KEY: H256 = Hasher::digest("metadata_root");
    static ref EPOCH_SEGMENT_KEY: H256 = Hasher::digest("epoch_segment");
    static ref CURRENT_METADATA_ROOT: ArcSwap<H256> = ArcSwap::from_pointee(H256::default());
    static ref METADATA_CACHE: RwLock<LruCache<Epoch, Metadata>> =  RwLock::new(LruCache::new(METADATA_CACHE_SIZE));
}

pub fn init<P: AsRef<Path>, B: Backend>(path: P, config: ConfigRocksDB, backend: Arc<B>) {
    let current_cell_root = backend.storage(MetadataContract::ADDRESS, *METADATA_ROOT_KEY);

    CURRENT_METADATA_ROOT.store(Arc::new(current_cell_root));

    METADATA_DB.get_or_init(|| {
        Arc::new(
            RocksTrieDB::new(path, config, METADATA_DB_CACHE_SIZE)
                .expect("[metadata] new rocksdb error"),
        )
    });
}

#[derive(Default)]
pub struct MetadataContract;

impl SystemContract for MetadataContract {
    const ADDRESS: H160 = system_contract_address(0x1);

    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp {
        let sender = tx.sender;
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();
        let gas_limit = *tx.gas_limit();
        let block_number = backend.block_number().as_u64();

        let mut store = exec_try!(
            MetadataStore::new(),
            gas_limit,
            "[metadata] init metadata mpt"
        );

        if block_number != 0 {
            let handle = MetadataHandle::default();

            if !exec_try!(
                handle.is_validator(block_number, sender),
                gas_limit,
                "[metadata] is validator"
            ) {
                return revert_resp(gas_limit);
            }
        }

        let call_abi = exec_try!(
            metadata_abi::MetadataContractCalls::decode(tx_data),
            gas_limit,
            "[metadata] invalid tx data"
        );

        match call_abi {
            metadata_abi::MetadataContractCalls::AppendMetadata(c) => {
                exec_try!(
                    store.append_metadata(&c.metadata.into()),
                    gas_limit,
                    "[metadata] append metadata"
                );
            }
            _ => unreachable!(),
        }

        update_mpt_root(backend);

        succeed_resp(gas_limit)
    }
}

fn update_mpt_root<B: Backend + ApplyBackend>(backend: &mut B) {
    let account = backend.basic(MetadataContract::ADDRESS);
    backend.apply(
        vec![Apply::Modify {
            address:       MetadataContract::ADDRESS,
            basic:         Basic {
                balance: account.balance,
                nonce:   account.nonce + U256::one(),
            },
            code:          None,
            storage:       vec![(*METADATA_ROOT_KEY, **CURRENT_METADATA_ROOT.load())],
            reset_storage: false,
        }],
        vec![],
        false,
    );
}
