mod convert;
mod metadata_abi;
mod segment;
mod store;

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
    Apply, Basic, ExitReason, ExitRevert, ExitSucceed, Hasher, Metadata, SignedTransaction, TxResp,
    H160, H256, U256,
};

use crate::system_contract::metadata::store::MetadataStore;
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

        let call_abi = match metadata_abi::MetadataContractCalls::decode(tx_data) {
            Ok(r) => r,
            Err(e) => {
                log::error!("[image cell] invalid tx data: {:?}", e);
                return revert_resp(*tx.gas_limit());
            }
        };

        match call_abi {
            metadata_abi::MetadataContractCalls::AppendMetadata(c) => {
                let mut store = match MetadataStore::new() {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("[metadata] init metadata mpt {:?}", e);
                        return revert_resp(*tx.gas_limit());
                    }
                };

                match store.append_metadata(c.metadata.into()) {
                    Ok(_) => (),
                    Err(e) => {
                        log::error!("[metadata] append metadata {:?}", e);
                        return revert_resp(*tx.gas_limit());
                    }
                }
            }
            _ => unreachable!(),
        }

        update_mpt_root(backend);

        TxResp {
            exit_reason:  ExitReason::Succeed(ExitSucceed::Returned),
            ret:          vec![],
            gas_used:     0u64,
            remain_gas:   tx.gas_limit().as_u64(),
            fee_cost:     U256::zero(),
            logs:         vec![],
            code_address: None,
            removed:      false,
        }
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

fn revert_resp(gas_limit: U256) -> TxResp {
    TxResp {
        exit_reason:  ExitReason::Revert(ExitRevert::Reverted),
        ret:          vec![],
        gas_used:     1u64,
        remain_gas:   (gas_limit - 1).as_u64(),
        fee_cost:     U256::one(),
        logs:         vec![],
        code_address: None,
        removed:      false,
    }
}
