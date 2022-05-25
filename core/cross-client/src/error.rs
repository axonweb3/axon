use protocol::Display;
use protocol::ProtocolError;
use protocol::ProtocolErrorKind;

#[derive(Clone, Debug, Display)]
pub enum CrossChainError {
    #[display(fmt = "rocksdb error {}", _0)]
    DB(rocksdb::Error),

    #[display(fmt = "create cross chain db error")]
    CreateDB,

    #[display(fmt = "batch length mismatch")]
    BatchLengthMismatch,
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
