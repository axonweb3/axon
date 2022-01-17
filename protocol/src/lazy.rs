use arc_swap::ArcSwap;

use crate::types::{MerkleRoot, H160};

lazy_static::lazy_static! {
    pub static ref CURRENT_STATE_ROOT: ArcSwap<MerkleRoot> = ArcSwap::from_pointee(Default::default());
    pub static ref CHAIN_ID: ArcSwap<u64> = ArcSwap::from_pointee(Default::default());
    pub static ref ASSET_CONTRACT_ADDRESS: ArcSwap<H160> = ArcSwap::from_pointee(Default::default());
}
