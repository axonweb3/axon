use arc_swap::ArcSwap;
use ckb_always_success_script::ALWAYS_SUCCESS;
use ckb_types::{bytes::Bytes, core::cell::CellMeta, core::ScriptHashType, packed, prelude::*};

use crate::traits::BYTE_SHANNONS;
use crate::{ckb_blake2b_256, types::Hex};

use std::sync::OnceLock;

static CELL: OnceLock<CellMeta> = OnceLock::new();

lazy_static::lazy_static! {
    pub static ref CHAIN_ID: ArcSwap<u64> = ArcSwap::from_pointee(Default::default());
    pub static ref PROTOCOL_VERSION: ArcSwap<Hex> = ArcSwap::from_pointee(Default::default());

    pub static ref ALWAYS_SUCCESS_DEPLOY_TX_HASH: [u8; 32] = ckb_blake2b_256("AlwaysSuccessDeployTx");
    pub static ref ALWAYS_SUCCESS_TYPE_SCRIPT: packed::Script
        = packed::ScriptBuilder::default()
            .code_hash(ckb_blake2b_256(ALWAYS_SUCCESS).pack())
            .hash_type(ScriptHashType::Data1.into())
            .build();
    pub static ref DUMMY_INPUT_OUT_POINT: packed::OutPoint
        = packed::OutPointBuilder::default()
            .tx_hash(ckb_blake2b_256("DummyInputOutpointTxHash").pack())
            .index(0u32.pack())
            .build();
}

pub fn always_success_script_meta() -> &'static CellMeta {
    CELL.get_or_init(|| {
        let capacity = (32 + 8 + 1 + ALWAYS_SUCCESS.len()) as u64 * BYTE_SHANNONS;
        let deploy_cell_output = packed::CellOutputBuilder::default()
            .capacity(capacity.pack())
            .build();
        let deploy_cell_out_point = packed::OutPointBuilder::default()
            .tx_hash(ALWAYS_SUCCESS_DEPLOY_TX_HASH.pack())
            .index(0u32.pack())
            .build();
        CellMeta {
            cell_output:        deploy_cell_output,
            out_point:          deploy_cell_out_point,
            transaction_info:   None,
            data_bytes:         ALWAYS_SUCCESS.len() as u64,
            mem_cell_data:      Some(Bytes::from(ALWAYS_SUCCESS.to_vec())),
            mem_cell_data_hash: Some(packed::Byte32::new_unchecked(Bytes::from(
                ckb_blake2b_256(ALWAYS_SUCCESS).to_vec(),
            ))),
        }
    })
}
