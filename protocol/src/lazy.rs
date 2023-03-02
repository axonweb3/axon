use arc_swap::ArcSwap;
use ckb_always_success_script::ALWAYS_SUCCESS;
use ckb_types::{packed, prelude::*};

use crate::types::{Hasher, MerkleRoot, Hex};

lazy_static::lazy_static! {
    pub static ref CURRENT_STATE_ROOT: ArcSwap<MerkleRoot> = ArcSwap::from_pointee(Default::default());
    pub static ref CHAIN_ID: ArcSwap<u64> = ArcSwap::from_pointee(Default::default());
    pub static ref PROTOCOL_VERSION: ArcSwap<Hex> = ArcSwap::from_pointee(Default::default());
    pub static ref CELL_VERIFIER_CODE_HASH: H256 = Hasher::digest("AxonCellVerifier");
    pub static ref ALWAYS_SUCCESS_CODE_HASH: [u8; 32] = ckb_hash::blake2b_256(ALWAYS_SUCCESS);
    pub static ref ALWAYS_SUCCESS_DEPLOY_TX_HASH: [u8; 32] = Hasher::digest("AlwaysSuccessDeployTx").0;
    pub static ref DUMMY_INPUT_OUT_POINT: packed::OutPoint
        = packed::OutPointBuilder::default()
            .tx_hash(Hasher::digest("DummyInputOutpointTxHash").0.pack())
            .index(0u32.pack()).build();
}
