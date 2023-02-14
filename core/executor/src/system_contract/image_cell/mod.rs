mod abi;
mod data_provider;
mod exec;
mod store;
mod trie_db;

pub use abi::image_cell_abi;
pub use data_provider::DataProvider;
pub use store::{CellInfo, CellKey};
pub(crate) use trie_db::RocksTrieDB;

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use arc_swap::ArcSwap;
use ckb_types::packed;
use ethers::abi::AbiDecode;
use once_cell::sync::OnceCell;

use common_config_parser::types::ConfigRocksDB;
use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{
    Apply, Basic, ExitReason, ExitRevert, ExitSucceed, Hasher, MerkleRoot, SignedTransaction,
    TxResp, H160, H256, U256,
};
use protocol::ProtocolResult;

use crate::system_contract::error::SystemScriptError;
use crate::system_contract::image_cell::store::{get_cell, get_header};
use crate::system_contract::{system_contract_address, SystemContract};
use crate::MPTTrie;

static ALLOW_READ: AtomicBool = AtomicBool::new(false);
static TRIE_DB: OnceCell<Arc<RocksTrieDB>> = OnceCell::new();

const DEFAULE_CACHE_SIZE: usize = 20;

lazy_static::lazy_static! {
    static ref CELL_ROOT_KEY: H256 = Hasher::digest("cell_mpt_root");
    static ref CURRENT_CELL_ROOT: ArcSwap<H256> = ArcSwap::from_pointee(H256::default());
}

pub fn init<P: AsRef<Path>, B: Backend>(path: P, config: ConfigRocksDB, backend: Arc<B>) {
    let current_cell_root = backend.storage(ImageCellContract::ADDRESS, *CELL_ROOT_KEY);

    CURRENT_CELL_ROOT.store(Arc::new(current_cell_root));

    TRIE_DB.get_or_init(|| {
        Arc::new(
            RocksTrieDB::new(path, config, DEFAULE_CACHE_SIZE)
                .expect("[image cell] new rocksdb error"),
        )
    });
}

#[derive(Default)]
pub struct ImageCellContract;

impl SystemContract for ImageCellContract {
    const ADDRESS: H160 = system_contract_address(0x1);

    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp {
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();

        match image_cell_abi::ImageCellCalls::decode(tx_data) {
            Ok(image_cell_abi::ImageCellCalls::SetState(data)) => {
                ALLOW_READ.store(data.allow_read, Ordering::Relaxed);
            }
            Ok(image_cell_abi::ImageCellCalls::Update(data)) => {
                let mut mpt = match get_mpt() {
                    Ok(m) => m,
                    Err(e) => {
                        log::error!("[image cell] get mpt error: {:?}", e);
                        return revert_resp(*tx.gas_limit());
                    }
                };

                let root: MerkleRoot = match exec::update(&mut mpt, data) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("[image cell] update error: {:?}", e);
                        return revert_resp(*tx.gas_limit());
                    }
                };

                update_mpt_root(backend, root);
            }
            Ok(image_cell_abi::ImageCellCalls::Rollback(data)) => {
                let mut mpt = match get_mpt() {
                    Ok(m) => m,
                    Err(e) => {
                        log::error!("[image cell] get mpt error: {:?}", e);
                        return revert_resp(*tx.gas_limit());
                    }
                };

                let root: MerkleRoot = match exec::rollback(&mut mpt, data) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("[image cell] rollback error: {:?}", e);
                        return revert_resp(*tx.gas_limit());
                    }
                };

                update_mpt_root(backend, root);
            }
            Err(e) => {
                log::error!("[image cell] invalid tx data: {:?}", e);
                return revert_resp(*tx.gas_limit());
            }
        }

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

fn get_mpt() -> ProtocolResult<MPTTrie<RocksTrieDB>> {
    let trie_db = match TRIE_DB.get() {
        Some(db) => db,
        None => return Err(SystemScriptError::TrieDbNotInit.into()),
    };

    let root = **CURRENT_CELL_ROOT.load();

    if root == H256::default() {
        Ok(MPTTrie::new(Arc::clone(trie_db)))
    } else {
        match MPTTrie::from_root(root, Arc::clone(trie_db)) {
            Ok(m) => Ok(m),
            Err(e) => Err(SystemScriptError::RestoreMpt(e.to_string()).into()),
        }
    }
}

fn update_mpt_root<B: Backend + ApplyBackend>(backend: &mut B, root: H256) {
    let account = backend.basic(ImageCellContract::ADDRESS);
    CURRENT_CELL_ROOT.swap(Arc::new(root));
    backend.apply(
        vec![Apply::Modify {
            address:       ImageCellContract::ADDRESS,
            basic:         Basic {
                balance: account.balance,
                nonce:   account.nonce + U256::one(),
            },
            code:          None,
            storage:       vec![(*CELL_ROOT_KEY, root)],
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

impl ImageCellContract {
    pub fn get_root(&self) -> H256 {
        **CURRENT_CELL_ROOT.load()
    }

    pub fn get_header(&self, key: &H256) -> ProtocolResult<Option<packed::Header>> {
        let mpt = get_mpt()?;
        get_header(&mpt, key)
    }

    pub fn get_cell(&self, key: &CellKey) -> ProtocolResult<Option<CellInfo>> {
        let mpt = get_mpt()?;
        get_cell(&mpt, key)
    }

    pub fn allow_read(&self) -> bool {
        ALLOW_READ.load(Ordering::Relaxed)
    }
}
