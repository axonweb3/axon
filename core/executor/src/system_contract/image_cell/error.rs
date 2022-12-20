use molecule::error::VerificationError;
use rlp::DecoderError;

use protocol::{Display, ProtocolError};

#[derive(Debug, Display)]
pub enum ImageCellError {
    #[display(fmt = "Invalid block number: {:?}", _0)]
    InvalidBlockNumber(u64),

    #[display(fmt = "RLP decoding failed: {:?}", _0)]
    RlpDecoder(DecoderError),

    #[display(fmt = "Protocol error: {:?}", _0)]
    Protocol(ProtocolError),

    #[display(fmt = "Molecule verification error: {:?}", _0)]
    MoleculeVerification(VerificationError),

    #[display(fmt = "TrieDB has not been initialized")]
    TrieDbNotInit,
}
