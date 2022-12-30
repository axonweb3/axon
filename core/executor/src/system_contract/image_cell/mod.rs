mod abi;
mod error;
mod exec;
mod store;
mod trie_db;

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ckb_types::packed;
use ethers::abi::AbiDecode;
use once_cell::sync::OnceCell;

use common_config_parser::types::ConfigRocksDB;
use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{
    Apply, Basic, ExitReason, ExitRevert, ExitSucceed, Hasher, MerkleRoot, SignedTransaction,
    TxResp, H160, H256, U256,
};

use crate::system_contract::{system_contract_address, SystemContract};
use crate::MPTTrie;

pub use abi::image_cell_abi;
pub use error::{ImageCellError, ImageCellResult};
pub use store::{cell_key, header_key, CellInfo, CellKey, HeaderKey};
use store::{get_block_number, get_cell, get_header};
use trie_db::RocksTrieDB;

static ALLOW_READ: AtomicBool = AtomicBool::new(false);
static TRIE_DB: OnceCell<Arc<RocksTrieDB>> = OnceCell::new();

lazy_static::lazy_static! {
    static ref CELL_ROOT_KEY: H256 = Hasher::digest("cell_mpt_root");
}

#[derive(Default)]
pub struct ImageCellContract;

pub fn init_image_cell<P: AsRef<Path>>(path: P, config: ConfigRocksDB, cache_size: usize){
    TRIE_DB.get_or_init(|| {
        Arc::new(
            RocksTrieDB::new(path, config, cache_size).expect("[image cell] new rocksdb error"),
        )
    });
}

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
                let mut mpt = match get_mpt(backend) {
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
                let mut mpt = match get_mpt(backend) {
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

fn get_mpt<B: Backend + ApplyBackend>(backend: &B) -> ImageCellResult<MPTTrie<RocksTrieDB>> {
    let trie_db = match TRIE_DB.get() {
        Some(db) => db,
        None => return Err(ImageCellError::TrieDbNotInit),
    };

    let root = backend.storage(ImageCellContract::ADDRESS, *CELL_ROOT_KEY);

    if root == H256::default() {
        Ok(MPTTrie::new(Arc::clone(trie_db)))
    } else {
        match MPTTrie::from_root(root, Arc::clone(trie_db)) {
            Ok(m) => Ok(m),
            Err(e) => Err(ImageCellError::RestoreMpt(e.to_string())),
        }
    }
}

fn update_mpt_root<B: Backend + ApplyBackend>(backend: &mut B, root: H256) {
    let account = backend.basic(ImageCellContract::ADDRESS);

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
    pub fn get_root<B: Backend + ApplyBackend>(&self, backend: &B) -> H256 {
        backend.storage(ImageCellContract::ADDRESS, *CELL_ROOT_KEY)
    }

    pub fn get_block_number<B: Backend + ApplyBackend>(
        &self,
        backend: &B,
    ) -> ImageCellResult<Option<u64>> {
        let mpt = get_mpt(backend)?;
        get_block_number(&mpt)
    }

    pub fn get_header<B: Backend + ApplyBackend>(
        &self,
        backend: &B,
        key: &HeaderKey,
    ) -> ImageCellResult<Option<packed::Header>> {
        let mpt = get_mpt(backend)?;
        get_header(&mpt, key)
    }

    pub fn get_cell<B: Backend + ApplyBackend>(
        &self,
        backend: &B,
        key: &CellKey,
    ) -> ImageCellResult<Option<CellInfo>> {
        let mpt = get_mpt(backend)?;
        get_cell(&mpt, key)
    }

    pub fn allow_read(&self) -> bool {
        ALLOW_READ.load(Ordering::Relaxed)
    }
}
