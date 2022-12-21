use std::error::Error;

use molecule::error::VerificationError;
use rlp::DecoderError;

use protocol::{Display, ProtocolError};

#[derive(Debug, Display)]
pub enum ImageCellError {
    #[display(fmt = "Invalid block number: {:?}", _0)]
    InvalidBlockNumber(u64),

    #[display(fmt = "Update block number error: {:?}, number: {:?}", e, number)]
    UpdateBlockNumber { e: ProtocolError, number: u64 },

    #[display(fmt = "Get block number error: {:?}", _0)]
    GetBlockNumber(ProtocolError),

    #[display(fmt = "Decode block number failed: {:?}", _0)]
    RlpDecodeBlockNumber(DecoderError),

    #[display(fmt = "Restore MPT error: {:?}", _0)]
    RestoreMpt(ProtocolError),

    #[display(fmt = "Insert cell error: {:?}", _0)]
    InsertCell(ProtocolError),

    #[display(fmt = "Remove cell error: {:?}", _0)]
    RemoveCell(ProtocolError),

    #[display(fmt = "Get cell error: {:?}", _0)]
    GetCell(ProtocolError),

    #[display(fmt = "Decode cell failed: {:?}", _0)]
    RlpDecodeCell(DecoderError),

    #[display(fmt = "Insert header error: {:?}", _0)]
    InsertHeader(ProtocolError),

    #[display(fmt = "Remove header error: {:?}", _0)]
    RemoveHeader(ProtocolError),

    #[display(fmt = "Get header error: {:?}", _0)]
    GetHeader(ProtocolError),

    #[display(fmt = "Commit error: {:?}", _0)]
    CommitError(ProtocolError),

    #[display(fmt = "Molecule verification error: {:?}", _0)]
    MoleculeVerification(VerificationError),

    #[display(fmt = "TrieDB has not been initialized")]
    TrieDbNotInit,
}

impl Error for ImageCellError {}

pub type ImageCellResult<T> = Result<T, ImageCellError>;
