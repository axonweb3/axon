use arc_swap::ArcSwap;
use ckb_types::{packed, prelude::*};

use crate::types::{Hasher, MerkleRoot, Hex};

lazy_static::lazy_static! {
    pub static ref CURRENT_STATE_ROOT: ArcSwap<MerkleRoot> = ArcSwap::from_pointee(Default::default());
    pub static ref CHAIN_ID: ArcSwap<u64> = ArcSwap::from_pointee(Default::default());
    pub static ref PROTOCOL_VERSION: ArcSwap<Hex> = ArcSwap::from_pointee(Default::default());
    pub static ref CELL_VERIFIER_CODE_HASH: H256 = Hasher::digest("AxonCellVerifier");
}
