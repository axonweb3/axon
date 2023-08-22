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

use std::sync::Arc;

use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::core::cell::{CellProvider, CellStatus};
use ckb_types::core::{HeaderBuilder, HeaderView};
use ckb_types::{packed, prelude::*};
use evm::backend::ApplyBackend;
use parking_lot::RwLock;
use rocksdb::DB;

use protocol::types::{Bytes, Hasher, SignedTransaction, TxResp, H160, H256};
use protocol::{ckb_blake2b_256, traits::ExecutorAdapter};

use crate::adapter::RocksTrieDB;
use crate::system_contract::{
    ckb_light_client::CkbHeaderReader, image_cell::ImageCellReader,
    utils::generate_mpt_root_changes,
};

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

#[macro_export]
macro_rules! system_contract_struct {
    ($name: ident) => {
        pub struct $name<Adapter: ExecutorAdapter + ApplyBackend>(
            std::marker::PhantomData<Adapter>,
        );

        impl<Adapter: ExecutorAdapter + ApplyBackend> Default for $name<Adapter> {
            fn default() -> Self {
                Self(std::marker::PhantomData)
            }
        }
    };
}

pub trait SystemContract<Adapter: ExecutorAdapter + ApplyBackend> {
    const ADDRESS: H160;

    fn exec_(&self, adapter: &mut Adapter, tx: &SignedTransaction) -> TxResp;

    fn before_block_hook(&self, _adapter: &mut Adapter) {}

    fn after_block_hook(&self, _adapter: &mut Adapter) {}
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

pub fn init<Adapter: ExecutorAdapter + ApplyBackend>(
    db: Arc<DB>,
    adapter: &mut Adapter,
) -> (H256, H256) {
    let current_metadata_root = adapter.storage(METADATA_CONTRACT_ADDRESS, *METADATA_ROOT_KEY);

    // Init metadata db.
    {
        let mut _db = METADATA_DB.write();
        _db.replace(Arc::new(RocksTrieDB::new_metadata(
            Arc::clone(&db),
            METADATA_DB_CACHE_SIZE,
        )));
    }

    {
        let mut _db = HEADER_CELL_DB.write();
        _db.replace(Arc::new(RocksTrieDB::new_ckb_light_client(
            db,
            HEADER_CELL_DB_CACHE_SIZE,
        )));
    }

    let current_cell_root =
        adapter.storage(CKB_LIGHT_CLIENT_CONTRACT_ADDRESS, *HEADER_CELL_ROOT_KEY);

    if current_cell_root.is_zero() {
        let changes = generate_mpt_root_changes(adapter, IMAGE_CELL_CONTRACT_ADDRESS);
        adapter.apply(changes, vec![], false);
    }
    (current_metadata_root, current_cell_root)
}

pub fn before_block_hook<Adapter: ExecutorAdapter + ApplyBackend>(adapter: &mut Adapter) {
    NativeTokenContract::default().before_block_hook(adapter);
    MetadataContract::default().before_block_hook(adapter);
    CkbLightClientContract::default().before_block_hook(adapter);
    ImageCellContract::default().before_block_hook(adapter);
}

pub fn after_block_hook<Adapter: ExecutorAdapter + ApplyBackend>(adapter: &mut Adapter) {
    NativeTokenContract::default().after_block_hook(adapter);
    MetadataContract::default().after_block_hook(adapter);
    CkbLightClientContract::default().after_block_hook(adapter);
    ImageCellContract::default().after_block_hook(adapter);
}

pub fn system_contract_dispatch<Adapter: ExecutorAdapter + ApplyBackend>(
    adapter: &mut Adapter,
    tx: &SignedTransaction,
) -> Option<TxResp> {
    if let Some(addr) = tx.get_to() {
        log::debug!("execute addr {:}", addr);

        if addr == NATIVE_TOKEN_CONTRACT_ADDRESS {
            return Some(NativeTokenContract::default().exec_(adapter, tx));
        } else if addr == METADATA_CONTRACT_ADDRESS {
            return Some(MetadataContract::default().exec_(adapter, tx));
        } else if addr == CKB_LIGHT_CLIENT_CONTRACT_ADDRESS {
            return Some(CkbLightClientContract::default().exec_(adapter, tx));
        } else if addr == IMAGE_CELL_CONTRACT_ADDRESS {
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
        if let Some(c) = ImageCellReader::default()
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
        ImageCellReader::default()
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
        CkbHeaderReader::default()
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
