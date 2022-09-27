use ibc::core::ics02_client::error::Error;

use protocol::{Display, ProtocolError, ProtocolErrorKind};

#[derive(Debug, Display)]
pub enum IbcError {
    Protocol(Error),

    #[display(fmt = "Adapter error {}", _0)]
    Adapter(String),
}

impl std::error::Error for IbcError {}

impl From<IbcError> for ProtocolError {
    fn from(err: IbcError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Ibc, Box::new(err))
    }
}
