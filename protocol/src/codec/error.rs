use std::error::Error;

use derive_more::Display;

use crate::{ProtocolError, ProtocolErrorKind};

#[derive(Debug, Display)]
pub enum CodecError {
    #[display(fmt = "rlp: from string {}", _0)]
    Rlp(String),
}

impl Error for CodecError {}

impl From<CodecError> for ProtocolError {
    fn from(err: CodecError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Codec, Box::new(err))
    }
}
