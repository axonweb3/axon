mod abi;
mod error;
mod exec;
mod store;
mod trie_db;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ckb_types::packed;
use ethers::abi::AbiDecode;
use once_cell::sync::OnceCell;

use common_config_parser::types::ConfigRocksDB;
use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{
    Apply, Basic, ExitReason, ExitRevert, ExitSucceed, MerkleRoot, SignedTransaction, TxResp, H160,
    H256, U256,
};

use crate::system_contract::{system_contract_address, SystemContract};
use crate::MPTTrie;

pub use abi::image_cell_abi;
pub use error::ImageCellError;
pub use store::{cell_key, header_key, CellInfo, CellKey, HeaderKey};
use store::{get_block_number, get_cell, get_header};
use trie_db::RocksTrieDB;

static ALLOW_READ: AtomicBool = AtomicBool::new(false);
static TRIE_DB: OnceCell<Arc<RocksTrieDB>> = OnceCell::new();

lazy_static::lazy_static! {
    static ref MPT_ROOT_KEY: H256 = H256::default();
    static ref H256_DEFAULT: H256 = H256::default();
}

pub struct MptConfig {
    pub path:          String,
    pub cache_size:    usize,
    pub rockdb_config: ConfigRocksDB,
}

pub struct ImageCellContract;

impl ImageCellContract {
    pub fn new(config: MptConfig) -> Self {
        TRIE_DB.get_or_init(|| {
            Arc::new(
                RocksTrieDB::new(config.path, config.rockdb_config, config.cache_size)
                    .expect("new rocksdb error"),
            )
        });
        ImageCellContract {}
    }
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
                        println!("{:?}", e);
                        return revert_resp(*tx.gas_limit());
                    }
                };

                let root: MerkleRoot = match exec::update(&mut mpt, data) {
                    Ok(r) => r,
                    Err(_) => return revert_resp(*tx.gas_limit()),
                };

                update_mpt_root(backend, root);
            }
            Ok(image_cell_abi::ImageCellCalls::Rollback(data)) => {
                let mut mpt = match get_mpt(backend) {
                    Ok(m) => m,
                    Err(_) => return revert_resp(*tx.gas_limit()),
                };

                let root: MerkleRoot = match exec::rollback(&mut mpt, data) {
                    Ok(r) => r,
                    Err(_) => return revert_resp(*tx.gas_limit()),
                };

                update_mpt_root(backend, root);
            }
            Err(_) => {
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

fn get_mpt<B: Backend + ApplyBackend>(backend: &B) -> Result<MPTTrie<RocksTrieDB>, ImageCellError> {
    let trie_db = match TRIE_DB.get() {
        Some(db) => db,
        None => return Err(ImageCellError::TrieDbNotInit),
    };

    let root = backend.storage(ImageCellContract::ADDRESS, *MPT_ROOT_KEY);

    if root == *H256_DEFAULT {
        Ok(MPTTrie::new(Arc::clone(trie_db)))
    } else {
        match MPTTrie::from_root(root, Arc::clone(trie_db)) {
            Ok(m) => Ok(m),
            Err(e) => Err(ImageCellError::Protocol(e)),
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
            storage:       vec![(*MPT_ROOT_KEY, root)],
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
        backend.storage(ImageCellContract::ADDRESS, *MPT_ROOT_KEY)
    }

    pub fn get_block_number<B: Backend + ApplyBackend>(
        &self,
        backend: &B,
    ) -> Result<Option<u64>, ImageCellError> {
        let mpt = get_mpt(backend)?;
        get_block_number(&mpt)
    }

    pub fn get_header<B: Backend + ApplyBackend>(
        &self,
        backend: &B,
        key: &HeaderKey,
    ) -> Result<Option<packed::Header>, ImageCellError> {
        let mpt = get_mpt(backend)?;
        get_header(&mpt, key)
    }

    pub fn get_cell<B: Backend + ApplyBackend>(
        &self,
        backend: &B,
        key: &CellKey,
    ) -> Result<Option<CellInfo>, ImageCellError> {
        let mpt = get_mpt(backend)?;
        get_cell(&mpt, key)
    }

    pub fn allow_read(&self) -> bool {
        ALLOW_READ.load(Ordering::Relaxed)
    }
}
