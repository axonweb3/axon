use ckb_always_success_script::ALWAYS_SUCCESS;
use protocol::lazy::ALWAYS_SUCCESS_DEPLOY_TX_HASH;

use super::image_cell_abi;

pub fn always_success_script_deploy_cell() -> image_cell_abi::CellInfo {
    image_cell_abi::CellInfo {
        out_point: image_cell_abi::OutPoint {
            tx_hash: *ALWAYS_SUCCESS_DEPLOY_TX_HASH,
            index:   0,
        },
        output:    Default::default(),
        data:      ALWAYS_SUCCESS.to_vec().into(),
    }
}
