#![cfg_attr(not(feature = "std"), no_std)]
// #![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

extern crate alloc;

mod error;
#[cfg(feature = "hash")]
pub mod hash;
#[cfg(feature = "hex")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "hex")))]
pub mod hex;
#[cfg(feature = "precompile")]
pub mod precompile;
#[cfg(all(feature = "proof", feature = "std"))]
mod proof;
mod rlp_codec;
pub mod types;

pub use error::Error;

#[cfg(feature = "proof")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "proof")))]
pub use proof::verify_proof;

#[cfg(all(feature = "proof", feature = "std"))]
pub use proof::verify_trie_proof;

#[cfg(feature = "hash")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "hash")))]
pub use hash::keccak_256;

pub mod consts;

#[cfg(test)]
mod tests;
