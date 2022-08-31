use protocol::{Display, ProtocolError, ProtocolErrorKind};

#[derive(Debug, Display)]
pub enum CrossChainError {
    #[display(fmt = "Rocksdb error {}", _0)]
    DB(sled::Error),

    #[display(fmt = "Adapter error {}", _0)]
    Adapter(String),

    #[display(fmt = "Create cross chain db error")]
    CreateDB,

    #[display(fmt = "Batch length mismatch")]
    BatchLengthMismatch,

    #[display(fmt = "Invalid cross direction")]
    InvalidDirection,

    #[display(fmt = "Sender error {}", _0)]
    Sender(String),

    #[display(fmt = "Crypto error {}", _0)]
    Crypto(String),
}

impl std::error::Error for CrossChainError {}

impl From<CrossChainError> for ProtocolError {
    fn from(err: CrossChainError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::CrossChain, Box::new(err))
    }
}

impl From<sled::Error> for CrossChainError {
    fn from(err: sled::Error) -> Self {
        CrossChainError::DB(err)
    }
}
