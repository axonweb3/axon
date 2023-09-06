pub mod codec;
pub mod lazy;
pub mod traits;
pub mod types;

use std::error::Error;

pub use derive_more::{Constructor, Display, From};
pub use {
    async_trait::async_trait, ckb_hash::blake2b_256 as ckb_blake2b_256, rand, thiserror, tokio,
    trie,
};

#[derive(Copy, Clone, Debug)]
pub enum ProtocolErrorKind {
    // traits
    API,
    Cli,
    CkbClient,
    Consensus,
    Contract,
    CrossChain,
    DB,
    Executor,
    Interoperation,
    Mempool,
    Network,
    Storage,
    Service,
    TxAssembler,
    Main,

    // types
    Types,
    Codec,

    // metric
    Metric,
}

// refer to https://github.com/rust-lang/rust/blob/a17951c4f80eb5208030f91fdb4ae93919fa6b12/src/libstd/io/error.rs#L73
#[derive(Debug, Constructor, Display)]
#[display(fmt = "[ProtocolError] Kind: {:?}, Error: {}", kind, error)]
pub struct ProtocolError {
    kind:  ProtocolErrorKind,
    error: Box<dyn Error + Send>,
}

impl From<ProtocolError> for Box<dyn Error + Send> {
    fn from(error: ProtocolError) -> Self {
        Box::new(error) as Box<dyn Error + Send>
    }
}

impl From<ProtocolError> for String {
    fn from(error: ProtocolError) -> String {
        error.to_string()
    }
}

impl From<trie::TrieError> for ProtocolError {
    fn from(error: trie::TrieError) -> Self {
        ProtocolError {
            kind:  ProtocolErrorKind::DB,
            error: Box::new(error),
        }
    }
}

impl Error for ProtocolError {}

pub type ProtocolResult<T> = Result<T, ProtocolError>;
