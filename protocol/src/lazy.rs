use arc_swap::ArcSwap;
use ckb_types::{packed, prelude::*};

use crate::{ckb_blake2b_256, types::Hex};

lazy_static::lazy_static! {
    pub static ref CHAIN_ID: ArcSwap<u64> = ArcSwap::from_pointee(Default::default());
    pub static ref PROTOCOL_VERSION: ArcSwap<Hex> = ArcSwap::from_pointee(Hex::with_length(8));

    pub static ref DUMMY_INPUT_OUT_POINT: packed::OutPoint
        = packed::OutPointBuilder::default()
            .tx_hash(ckb_blake2b_256("DummyInputOutpointTxHash").pack())
            .index(0u32.pack())
            .build();
}
