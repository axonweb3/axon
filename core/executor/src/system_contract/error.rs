use std::io;

use thiserror::Error;

use protocol::{ProtocolError, ProtocolErrorKind};

#[derive(Error, Debug)]
pub enum SystemScriptError {
    #[error("Create DB path {0}")]
    CreateDB(io::Error),

    #[error("rocksdb {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("Invalid block number: {0}")]
    InvalidBlockNumber(u64),

    #[error("Update block number error: {e:?}, number: {number:?}")]
    UpdateBlockNumber { e: String, number: u64 },

    #[error("Get block number error: {0}")]
    GetBlockNumber(String),

    #[error("Decode block number failed: {0}")]
    DecodeBlockNumber(rlp::DecoderError),

    #[error("Restore MPT error: {0}")]
    RestoreMpt(String),

    #[error("Insert cell error: {0}")]
    InsertCell(String),

    #[error("Remove cell error: {0}")]
    RemoveCell(String),

    #[error("Get cell error: {0}")]
    GetCell(String),

    #[error("Decode cell failed: {0}")]
    DecodeCell(rlp::DecoderError),

    #[error("Insert header error: {0}")]
    InsertHeader(String),

    #[error("Remove header error: {0}")]
    RemoveHeader(String),

    #[error("Get header error: {0}")]
    GetHeader(String),

    #[error("Commit error: {0}")]
    CommitError(String),

    #[error("Molecule verification error: {0}")]
    MoleculeVerification(#[from] molecule::error::VerificationError),

    #[error("TrieDB has not been initialized")]
    TrieDbNotInit,

    #[error("Data length mismatch expect {expect:?}, actual: {actual:?}")]
    DataLengthMismatch { expect: usize, actual: usize },

    #[error("Query for future epoch")]
    FutureEpoch,

    #[error("Decode epoch segment error {0}")]
    DecodeEpochSegment(String),

    #[error("Invalid epoch end {0}")]
    InvalidEpochEnd(u64),

    #[error("Add for past epoch")]
    PastEpoch,
}

impl From<SystemScriptError> for ProtocolError {
    fn from(error: SystemScriptError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Executor, Box::new(error))
    }
}
