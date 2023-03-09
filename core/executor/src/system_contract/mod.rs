mod error;

pub mod ckb_light_client;
pub mod image_cell;
pub mod metadata;
mod native_token;
mod trie_db;
mod utils;

use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::core::cell::{CellProvider, CellStatus};
use ckb_types::{core::HeaderView, packed, prelude::*};

pub use crate::system_contract::ckb_light_client::CkbLightClientContract;
pub use crate::system_contract::image_cell::ImageCellContract;
pub use crate::system_contract::metadata::MetadataContract;
pub use crate::system_contract::native_token::NativeTokenContract;

use protocol::ckb_blake2b_256;
use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{Bytes, SignedTransaction, TxResp, H160, H256};

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
