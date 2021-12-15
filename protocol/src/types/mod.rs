pub use batch::*;
pub use block::*;
pub use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use executor::{
    AccessList, AccessListItem, Account, Config, ExecResp, ExecutorContext, ExitReason, TxResp,
};
pub use primitive::*;
pub use receipt::*;
pub use transaction::*;

pub mod batch;
pub mod block;
pub mod executor;
pub mod primitive;
pub mod receipt;
pub mod transaction;

use std::error::Error;

use derive_more::{Display, From};

use crate::{ProtocolError, ProtocolErrorKind};

#[derive(Debug, Display, From)]
pub enum TypesError {
    #[display(fmt = "Expect {:?}, get {:?}.", expect, real)]
    LengthMismatch { expect: usize, real: usize },

    #[display(fmt = "{:?}", error)]
    FromHex { error: hex::FromHexError },

    #[display(fmt = "{:?} is an invalid address", address)]
    InvalidAddress { address: String },

    #[display(fmt = "Hex should start with 0x")]
    HexPrefix,

    #[display(fmt = "Invalid public key")]
    InvalidPublicKey,

    #[display(fmt = "Invalid check sum")]
    InvalidCheckSum,
}

impl Error for TypesError {}

impl From<TypesError> for ProtocolError {
    fn from(error: TypesError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Types, Box::new(error))
    }
}
