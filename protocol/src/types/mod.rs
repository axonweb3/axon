pub use ethereum::Log;

pub use batch::*;
pub use block::*;
pub use crosschain::*;
pub use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use ckb_client::*;
pub use evm::{backend::*, ExitRevert, ExitSucceed};
pub use executor::{
    AccessList, AccessListItem, Account, Config, ExecResp, ExecutorContext, ExitReason, TxResp,
};
pub use interoperation::VMResp;
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
pub mod crosschain;

use std::error::Error;

use derive_more::{Display, From};

use common_crypto::Error as CryptoError;

use crate::{ProtocolError, ProtocolErrorKind};

#[derive(Debug, Display, From)]
pub enum TypesError {
    #[display(fmt = "Expect {:?}, get {:?}.", expect, real)]
    LengthMismatch { expect: usize, real: usize },

    #[display(
        fmt = "Transaction hash mismatch origin {:?}, computed {:?}",
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

    #[display(fmt = "Invalid crosschain direction")]
    InvalidDirection,
}

impl Error for TypesError {}

impl From<TypesError> for ProtocolError {
    fn from(error: TypesError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Types, Box::new(error))
    }
}
