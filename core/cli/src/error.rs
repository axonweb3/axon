use std::io;

use common_version::Version;
use thiserror::Error;

use protocol::ProtocolError;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    // Boxing so the error type isn't too large (clippy::result-large-err).
    #[error(transparent)]
    CheckingVersion(Box<CheckingVersionError>),
    #[error("reading data version: {0}")]
    ReadingVersion(#[source] io::Error),
    #[error("writing data version: {0}")]
    WritingVersion(#[source] io::Error),

    #[error(transparent)]
    Running(ProtocolError),
}

#[non_exhaustive]
#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[error("data version({data}) is not compatible with the current axon version({current}), version >= {least_compatible} is supported")]
pub struct CheckingVersionError {
    pub current:          Version,
    pub data:             Version,
    pub least_compatible: Version,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
