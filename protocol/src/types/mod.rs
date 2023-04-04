pub use ethereum::Log;

pub use batch::*;
pub use block::*;
pub use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use ckb_client::*;
pub use evm::{backend::*, ExitError, ExitRevert, ExitSucceed};
pub use executor::{
    AccessList, AccessListItem, Account, Config, ExecResp, ExecutorContext, ExitReason, TxResp,
};
pub use interoperation::*;
pub use primitive::*;
pub use receipt::*;
pub use transaction::*;

pub mod batch;
pub mod block;
pub mod ckb_client;
pub mod executor;
pub mod interoperation;
pub mod primitive;
pub mod receipt;
pub mod transaction;

use std::error::Error;

use derive_more::{Display, From};

use common_crypto::Error as CryptoError;

use crate::{ProtocolError, ProtocolErrorKind};

#[derive(Debug, Display, From)]
pub enum TypesError {
    #[display(fmt = "Expect {:?}, get {:?}.", expect, real)]
    LengthMismatch { expect: usize, real: usize },

    #[display(
        fmt = "Eip1559Transaction hash mismatch origin {:?}, computed {:?}",
        origin,
        calc
    )]
    TxHashMismatch { origin: H256, calc: H256 },

    #[display(fmt = "{:?}", _0)]
    FromHex(faster_hex::Error),

    #[display(fmt = "{:?} is an invalid address", _0)]
    InvalidAddress(String),

    #[display(fmt = "Hex should start with 0x")]
    HexPrefix,

    #[display(fmt = "Invalid public key")]
    InvalidPublicKey,

    #[display(fmt = "Invalid check sum")]
    InvalidCheckSum,

    #[display(fmt = "Unsigned")]
    Unsigned,

    #[display(fmt = "Crypto error {:?}", _0)]
    Crypto(CryptoError),

    #[display(fmt = "Missing signature")]
    MissingSignature,

    #[display(fmt = "Invalid crosschain direction")]
    InvalidDirection,

    #[display(fmt = "Signature R is empty")]
    SignatureRIsEmpty,

    #[display(fmt = "Invalid signature R type")]
    InvalidSignatureRType,

    #[display(fmt = "Invalid address source type")]
    InvalidAddressSourceType,

    #[display(fmt = "Missing interoperation sender")]
    MissingInteroperationSender,
}

impl Error for TypesError {}

impl From<TypesError> for ProtocolError {
    fn from(error: TypesError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Types, Box::new(error))
    }
}
