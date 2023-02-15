mod convert;
mod handle;
mod metadata_abi;
mod segment;
mod store;

pub use handle::MetadataHandle;

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

use crate::system_contract::{
    image_cell::RocksTrieDB, revert_resp, succeed_resp, system_contract_address, SystemContract,
};
use crate::{exec_try, system_contract::metadata::store::MetadataStore};

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

pub fn init<P: AsRef<Path>>(path: P, config: ConfigRocksDB) {
    METADATA_DB.get_or_init(|| {
        Arc::new(
            RocksTrieDB::new(path, config, METADATA_DB_CACHE_SIZE)
                .expect("[image cell] new rocksdb error"),
        )
    });
}

#[derive(Default)]
pub struct MetadataContract;

impl SystemContract for MetadataContract {
    const ADDRESS: H160 = system_contract_address(0x00);

    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp {
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();
        let gas_limit = *tx.gas_limit();
        let block_number = backend.block_number().as_u64();
        let sender = backend.origin();

        let mut store = exec_try!(
            MetadataStore::new(),
            gas_limit,
            "[metadata] init metadata mpt"
        );
        let epoch_segment = exec_try!(
            store.get_epoch_segment(),
            gas_limit,
            "[metadata] get epoch segment"
        );
        let epoch_number = exec_try!(
            epoch_segment.get_epoch_number(block_number),
            gas_limit,
            "get_epoch"
        );
        let current_metadata =
            exec_try!(store.get_metadata(epoch_number), gas_limit, "get_metadata");

        if current_metadata
            .verifier_list
            .iter()
            .all(|v| v.address != sender)
        {
            log::error!("[metadata]: invalid sender");
            return revert_resp(gas_limit);
        }

        let call_abi = exec_try!(
            metadata_abi::MetadataContractCalls::decode(tx_data),
            gas_limit,
            "[metadata] invalid tx data"
        );

        match call_abi {
            metadata_abi::MetadataContractCalls::AppendMetadata(c) => {
                exec_try!(
                    store.append_metadata(c.metadata.into()),
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
