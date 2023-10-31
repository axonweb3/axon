#![cfg_attr(doc_cfg, feature(doc_cfg))]

extern crate alloc;

mod error;
mod hash;
mod proof;
pub mod types;

#[cfg(feature = "proof")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "proof")))]
pub use proof::{verify_proof, verify_trie_proof};
