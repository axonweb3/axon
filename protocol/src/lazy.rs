use arc_swap::ArcSwap;
use ckb_always_success_script::ALWAYS_SUCCESS;
use ckb_types::{core::ScriptHashType, packed, prelude::*};

use crate::{ckb_blake2b_256, types::Hex};

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
