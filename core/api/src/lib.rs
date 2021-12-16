pub mod adapter;
pub mod graphql;
pub mod jsonrpc;

pub use adapter::DefaultAPIAdapter;

use std::error::Error;

use protocol::{Display, ProtocolError, ProtocolErrorKind};

#[derive(Debug, Display)]
pub enum APIError {
    #[display(fmt = "adapter error {:?}", _0)]
    AdapterError(String),

    #[display(fmt = "http server error {:?}", _0)]
    HttpServer(String),
}

impl Error for APIError {}

impl From<APIError> for ProtocolError {
    fn from(error: APIError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::API, Box::new(error))
    }
}
