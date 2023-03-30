pub mod image_cell_abi;

use ckb_types::{packed, prelude::Unpack};

use protocol::types::OutPoint;

impl From<OutPoint> for image_cell_abi::OutPoint {
    fn from(value: OutPoint) -> Self {
        image_cell_abi::OutPoint {
            tx_hash: value.tx_hash.0,
            index:   value.index,
        }
    }
}

impl From<packed::CellOutput> for image_cell_abi::CellOutput {
    fn from(value: packed::CellOutput) -> Self {
        image_cell_abi::CellOutput {
            capacity: value.capacity().unpack(),
            lock:     value.lock().into(),
            type_:    value
                .type_()
                .to_opt()
                .map(|v| vec![v.into()])
                .unwrap_or_default(),
        }
    }
}

impl From<packed::Script> for image_cell_abi::Script {
    fn from(value: packed::Script) -> Self {
        let args: Vec<u8> = value.args().unpack();

        image_cell_abi::Script {
            code_hash: value.code_hash().unpack().0,
            hash_type: value.hash_type().into(),
            args:      args.into(),
        }
    }
}
