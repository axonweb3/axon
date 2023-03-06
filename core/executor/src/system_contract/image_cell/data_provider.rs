use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::core::cell::{CellProvider, CellStatus};
use ckb_types::{core::HeaderView, packed, prelude::*};

use protocol::ckb_blake2b_256;
use protocol::types::{Bytes, H256};

use crate::system_contract::image_cell::ImageCellContract;

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
        ImageCellContract::default()
            .get_header(&H256(tmp.0))
            .ok()
            .flatten()
            .map(|h| h.into_view())
    }
}
