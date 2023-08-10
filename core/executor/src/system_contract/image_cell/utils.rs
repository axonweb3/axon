use ckb_always_success_script::ALWAYS_SUCCESS;
use ckb_types::{packed, prelude::*};

use protocol::{lazy::ALWAYS_SUCCESS_DEPLOY_TX_HASH, traits::BYTE_SHANNONS};

use crate::system_contract::image_cell::image_cell_abi;

pub fn always_success_script_deploy_cell() -> image_cell_abi::CellInfo {
    let capacity = (32 + 8 + 1 + ALWAYS_SUCCESS.len()) as u64 * BYTE_SHANNONS;

    image_cell_abi::CellInfo {
        out_point: image_cell_abi::OutPoint {
            tx_hash: *ALWAYS_SUCCESS_DEPLOY_TX_HASH,
            index:   0,
        },
        output:    packed::CellOutputBuilder::default()
            .capacity(capacity.pack())
            .build()
            .into(),
        data:      ALWAYS_SUCCESS.to_vec().into(),
    }
}
