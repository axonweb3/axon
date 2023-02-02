pub mod image_cell_abi;

use protocol::types::OutPoint;

impl From<OutPoint> for image_cell_abi::OutPoint {
    fn from(value: OutPoint) -> Self {
        image_cell_abi::OutPoint {
            tx_hash: value.tx_hash.0,
            index:   value.index,
        }
    }
}
