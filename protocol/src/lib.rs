#![allow(clippy::mutable_key_type, clippy::derive_hash_xor_eq, dead_code)]

pub mod codec;
pub mod traits;
pub mod types;

use std::error::Error;

pub use async_trait::async_trait;
pub use derive_more::{Constructor, Display, From};
pub use tokio;

#[derive(Debug, Clone)]
pub enum ProtocolErrorKind {
    // traits
    API,
    Consensus,
    Executor,
    Mempool,
    Network,
    Storage,
    Service,
    Main,

    // types
    Types,
    Codec,

    // metric
    Metric,
    Cli,
}

// refer to https://github.com/rust-lang/rust/blob/a17951c4f80eb5208030f91fdb4ae93919fa6b12/src/libstd/io/error.rs#L73
#[derive(Debug, Constructor, Display)]
#[display(fmt = "[ProtocolError] Kind: {:?} Error: {:?}", kind, error)]
pub struct ProtocolError {
    kind:  ProtocolErrorKind,
    error: Box<dyn Error + Send>,
}

impl From<ProtocolError> for Box<dyn Error + Send> {
    fn from(error: ProtocolError) -> Self {
        Box::new(error) as Box<dyn Error + Send>
    }
}

impl Error for ProtocolError {}

pub type ProtocolResult<T> = Result<T, ProtocolError>;
