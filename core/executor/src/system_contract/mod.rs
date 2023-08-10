mod error;
mod native_token;
mod utils;

pub(crate) mod ckb_light_client;
pub(crate) mod image_cell;
pub mod metadata;

pub use crate::system_contract::ckb_light_client::{
    CkbLightClientContract, CKB_LIGHT_CLIENT_CONTRACT_ADDRESS,
};
pub use crate::system_contract::image_cell::{ImageCellContract, IMAGE_CELL_CONTRACT_ADDRESS};
pub use crate::system_contract::metadata::{
    check_ckb_related_info_exist, MetadataContract, METADATA_CONTRACT_ADDRESS,
};
pub use crate::system_contract::native_token::{
    NativeTokenContract, NATIVE_TOKEN_CONTRACT_ADDRESS,
};

use std::path::Path;
use std::sync::Arc;

use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::core::cell::{CellProvider, CellStatus};
use ckb_types::core::{HeaderBuilder, HeaderView};
use ckb_types::{packed, prelude::*};
use parking_lot::RwLock;

use common_config_parser::types::ConfigRocksDB;
use protocol::types::{Bytes, Hasher, SignedTransaction, TxResp, H160, H256};
use protocol::{ckb_blake2b_256, traits::ExecutorAdapter};

use crate::adapter::RocksTrieDB;
use crate::system_contract::image_cell::utils::always_success_script_deploy_cell;
use crate::system_contract::utils::generate_mpt_root_changes;

pub const fn system_contract_address(addr: u8) -> H160 {
    H160([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, addr,
    ])
}
const HEADER_CELL_DB_CACHE_SIZE: usize = 200;
const METADATA_DB_CACHE_SIZE: usize = 10;

lazy_static::lazy_static! {
    pub static ref HEADER_CELL_ROOT_KEY: H256 = Hasher::digest("header_cell_mpt_root");
    pub static ref METADATA_ROOT_KEY: H256 = Hasher::digest("metadata_root");
    pub(crate) static ref METADATA_DB: RwLock<Option<Arc<RocksTrieDB>>> = RwLock::new(None);
    pub(crate) static ref HEADER_CELL_DB: RwLock<Option<Arc<RocksTrieDB>>> = RwLock::new(None);
}

#[macro_export]
macro_rules! exec_try {
    ($func: expr, $gas_limit: expr, $log_msg: literal) => {
        match $func {
            Ok(r) => r,
            Err(e) => {
                log::error!("{:?} {:?}", $log_msg, e);
                return $crate::system_contract::utils::revert_resp($gas_limit);
            }
        }
    };
}

pub trait SystemContract {
    const ADDRESS: H160;

    fn exec_<Adapter: ExecutorAdapter>(
        &self,
        adapter: &mut Adapter,
        tx: &SignedTransaction,
    ) -> TxResp;

    fn before_block_hook<Adapter: ExecutorAdapter>(&self, _adapter: &mut Adapter) {}

    fn after_block_hook<Adapter: ExecutorAdapter>(&self, _adapter: &mut Adapter) {}
}

pub fn swap_metadata_db(new_db: Arc<RocksTrieDB>) -> Arc<RocksTrieDB> {
    METADATA_DB
        .write()
        .replace(new_db)
        .unwrap_or_else(|| panic!("metadata db is not initialized"))
}

pub fn swap_header_cell_db(new_db: Arc<RocksTrieDB>) -> Arc<RocksTrieDB> {
    HEADER_CELL_DB
        .write()
        .replace(new_db)
        .unwrap_or_else(|| panic!("header cell db is not initialized"))
}

pub fn init<P: AsRef<Path>, Adapter: ExecutorAdapter>(
    path: P,
    config: ConfigRocksDB,
    adapter: &mut Adapter,
) -> (H256, H256) {
    let current_metadata_root = adapter.storage(MetadataContract::ADDRESS, *METADATA_ROOT_KEY);

    // Init metadata db.
    let metadata_db_path = path.as_ref().join("metadata");
    {
        let mut db = METADATA_DB.write();
        db.replace(Arc::new(
            RocksTrieDB::new(metadata_db_path, config.clone(), METADATA_DB_CACHE_SIZE)
                .expect("[system contract] metadata new rocksdb error"),
        ));
    }

    let header_cell_db_path = path.as_ref().join("header_cell");
    {
        let mut db = HEADER_CELL_DB.write();
        db.replace(Arc::new(
            RocksTrieDB::new(header_cell_db_path, config, HEADER_CELL_DB_CACHE_SIZE)
                .expect("[system contract] header&cell new rocksdb error"),
        ));
    }

    let current_cell_root = adapter.storage(CkbLightClientContract::ADDRESS, *HEADER_CELL_ROOT_KEY);

    if current_cell_root.is_zero() {
        // todo need refactoring
        ImageCellContract::default()
            .save_cells(H256::zero(), vec![always_success_script_deploy_cell()], 0)
            .unwrap();
        let changes = generate_mpt_root_changes(adapter, ImageCellContract::ADDRESS);
        adapter.apply(changes, vec![], false);
    }

    (current_metadata_root, current_cell_root)
}

pub fn before_block_hook<Adapter: ExecutorAdapter>(adapter: &mut Adapter) {
    NativeTokenContract::default().before_block_hook(adapter);
    MetadataContract::default().before_block_hook(adapter);
    CkbLightClientContract::default().before_block_hook(adapter);
    ImageCellContract::default().before_block_hook(adapter);
}

pub fn after_block_hook<Adapter: ExecutorAdapter>(adapter: &mut Adapter) {
    NativeTokenContract::default().after_block_hook(adapter);
    MetadataContract::default().after_block_hook(adapter);
    CkbLightClientContract::default().after_block_hook(adapter);
    ImageCellContract::default().after_block_hook(adapter);
}

pub fn system_contract_dispatch<Adapter: ExecutorAdapter>(
    adapter: &mut Adapter,
    tx: &SignedTransaction,
) -> Option<TxResp> {
    if let Some(addr) = tx.get_to() {
        log::debug!("execute addr {:}", addr);

        if addr == NativeTokenContract::ADDRESS {
            return Some(NativeTokenContract::default().exec_(adapter, tx));
        } else if addr == MetadataContract::ADDRESS {
            return Some(MetadataContract::default().exec_(adapter, tx));
        } else if addr == CkbLightClientContract::ADDRESS {
            return Some(CkbLightClientContract::default().exec_(adapter, tx));
        } else if addr == ImageCellContract::ADDRESS {
            return Some(ImageCellContract::default().exec_(adapter, tx));
        }
    }
    None
}

#[derive(Clone, Debug)]
pub struct DataProvider {
    root: H256,
}

impl CellProvider for DataProvider {
    fn cell(&self, out_point: &packed::OutPoint, _eager_load: bool) -> CellStatus {
        if let Some(c) = ImageCellContract::default()
            .get_cell(self.root, &(out_point).into())
            .ok()
            .flatten()
        {
            return CellStatus::Live(c.into_meta(out_point));
        }

        CellStatus::Unknown
    }
}

impl CellDataProvider for DataProvider {
    fn get_cell_data(&self, out_point: &packed::OutPoint) -> Option<Bytes> {
        ImageCellContract::default()
            .get_cell(self.root, &(out_point.into()))
            .ok()
            .flatten()
            .map(|info| info.cell_data)
    }

    fn get_cell_data_hash(&self, out_point: &packed::OutPoint) -> Option<packed::Byte32> {
        self.get_cell_data(out_point).map(|data| {
            if data.is_empty() {
                packed::Byte32::zero()
            } else {
                ckb_blake2b_256(data).pack()
            }
        })
    }
}

impl HeaderProvider for DataProvider {
    fn get_header(&self, hash: &packed::Byte32) -> Option<HeaderView> {
        let block_hash = hash.unpack();
        CkbLightClientContract::default()
            .get_header_by_block_hash(self.root, &H256(block_hash.0))
            .ok()
            .flatten()
            .map(|h| {
                HeaderBuilder::default()
                    .version(h.version.pack())
                    .parent_hash(h.parent_hash.pack())
                    .timestamp(h.timestamp.pack())
                    .number(h.number.pack())
                    .epoch(h.epoch.pack())
                    .transactions_root(h.transactions_root.pack())
                    .proposals_hash(h.proposals_hash.pack())
                    .extra_hash(h.extra_hash.pack())
                    .compact_target(h.compact_target.pack())
                    .dao(h.dao.pack())
                    .nonce(h.nonce.pack())
                    .build()
            })
    }
}

impl DataProvider {
    pub fn new(root: H256) -> Self {
        DataProvider { root }
    }
}
