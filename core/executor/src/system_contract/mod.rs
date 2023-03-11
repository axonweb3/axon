mod error;
mod native_token;
mod trie_db;
mod utils;

pub mod ckb_light_client;
pub mod image_cell;
pub mod metadata;

use std::path::Path;
use std::sync::Arc;

use arc_swap::ArcSwap;
use common_config_parser::types::ConfigRocksDB;
use once_cell::sync::OnceCell;

use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::core::cell::{CellProvider, CellStatus};
use ckb_types::core::HeaderView;
use ckb_types::{packed, prelude::*};

use crate::system_contract::image_cell::utils::always_success_script_deploy_cell;
use crate::system_contract::trie_db::RocksTrieDB;
use crate::system_contract::utils::update_mpt_root;

use protocol::ckb_blake2b_256;
use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{Bytes, Hasher, SignedTransaction, TxResp, H160, H256};

pub use crate::system_contract::ckb_light_client::CkbLightClientContract;
pub use crate::system_contract::image_cell::ImageCellContract;
pub use crate::system_contract::metadata::MetadataContract;
pub use crate::system_contract::native_token::NativeTokenContract;

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

pub const fn system_contract_address(addr: u8) -> H160 {
    H160([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, addr,
    ])
}

/// system contract init section
/// need to initialize two databases, one for Metadata and one for
/// CkbLightClient&ImageCell
static HEADER_CELL_DB: OnceCell<Arc<RocksTrieDB>> = OnceCell::new();
static METADATA_DB: OnceCell<Arc<RocksTrieDB>> = OnceCell::new();

const HEADER_CELL_DB_CACHE_SIZE: usize = 20;
const METADATA_DB_CACHE_SIZE: usize = 20;

lazy_static::lazy_static! {
    pub static ref HEADER_CELL_ROOT_KEY: H256 = Hasher::digest("header_cell_mpt_root");
    static ref CURRENT_HEADER_CELL_ROOT: ArcSwap<H256> = ArcSwap::from_pointee(H256::default());
    static ref METADATA_ROOT_KEY: H256 = Hasher::digest("metadata_root");
    static ref CURRENT_METADATA_ROOT: ArcSwap<H256> = ArcSwap::from_pointee(H256::default());
}

pub fn init<P: AsRef<Path>, B: Backend + ApplyBackend>(
    path: P,
    config: ConfigRocksDB,
    mut backend: B,
) {
    // init metadata db
    let current_metadata_root = backend.storage(MetadataContract::ADDRESS, *METADATA_ROOT_KEY);
    CURRENT_METADATA_ROOT.store(Arc::new(current_metadata_root));

    let metadata_db_path = path.as_ref().join("metadata");
    METADATA_DB.get_or_init(|| {
        Arc::new(
            RocksTrieDB::new(metadata_db_path, config.clone(), METADATA_DB_CACHE_SIZE)
                .expect("[system contract] metadata new rocksdb error"),
        )
    });

    // init header cell db
    let header_cell_db_path = path.as_ref().join("header_cell");
    HEADER_CELL_DB.get_or_init(|| {
        Arc::new(
            RocksTrieDB::new(
                header_cell_db_path,
                config.clone(),
                HEADER_CELL_DB_CACHE_SIZE,
            )
            .expect("[system contract] header&cell new rocksdb error"),
        )
    });

    let current_cell_root = backend.storage(CkbLightClientContract::ADDRESS, *HEADER_CELL_ROOT_KEY);

    if current_cell_root.is_zero() {
        // todo need refactoring
        ImageCellContract::default()
            .save_cells(vec![always_success_script_deploy_cell()], 0)
            .unwrap();
        return update_mpt_root(&mut backend, CkbLightClientContract::ADDRESS);
    }

    CURRENT_HEADER_CELL_ROOT.store(Arc::new(current_cell_root));
}

pub trait SystemContract {
    const ADDRESS: H160;

    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp;
}

pub fn system_contract_dispatch<B: Backend + ApplyBackend>(
    backend: &mut B,
    tx: &SignedTransaction,
) -> Option<TxResp> {
    if let Some(addr) = tx.get_to() {
        if addr == NativeTokenContract::ADDRESS {
            return Some(NativeTokenContract::default().exec_(backend, tx));
        } else if addr == MetadataContract::ADDRESS {
            return Some(MetadataContract::default().exec_(backend, tx));
        } else if addr == CkbLightClientContract::ADDRESS {
            return Some(CkbLightClientContract::default().exec_(backend, tx));
        } else if addr == ImageCellContract::ADDRESS {
            return Some(ImageCellContract::default().exec_(backend, tx));
        }
    }

    None
}

#[derive(Default, Clone)]
pub struct DataProvider;

impl CellProvider for DataProvider {
    fn cell(&self, out_point: &packed::OutPoint, _eager_load: bool) -> CellStatus {
        if let Some(c) = ImageCellContract::default()
            .get_cell(&(out_point).into())
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
            .get_cell(&(out_point.into()))
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
        let tmp: ckb_types::H256 = hash.unpack();
        CkbLightClientContract::default()
            .get_header(&H256(tmp.0))
            .ok()
            .flatten()
            .map(|h| h.into_view())
    }
}
