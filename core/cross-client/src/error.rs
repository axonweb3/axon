use protocol::Display;
use protocol::ProtocolError;
use protocol::ProtocolErrorKind;

#[derive(Clone, Debug, Display)]
pub enum CrossChainError {
    #[display(fmt = "Rocksdb error {}", _0)]
    DB(rocksdb::Error),

    #[display(fmt = "Adapter error {}", _0)]
    Adapter(String),

    #[display(fmt = "Create cross chain db error")]
    CreateDB,

    #[display(fmt = "Batch length mismatch")]
    BatchLengthMismatch,

    #[display(fmt = "Invalid cross direction")]
    InvalidDirection,
}

impl std::error::Error for CrossChainError {}

impl From<CrossChainError> for ProtocolError {
    fn from(err: CrossChainError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::CrossChain, Box::new(err))
    }
}

impl From<rocksdb::Error> for CrossChainError {
    fn from(err: rocksdb::Error) -> Self {
        CrossChainError::DB(err)
    }
}
