use std::error::Error;

use derive_more::{Display, From};

use crate::{ProtocolError, ProtocolErrorKind};

#[derive(Debug, From, Display)]
pub enum CodecError {
    #[display(fmt = "from string {}", _0)]
    Rlp(String),
}

impl Error for CodecError {}

// TODO: derive macro
impl From<CodecError> for ProtocolError {
    fn from(err: CodecError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Codec, Box::new(err))
    }
}
