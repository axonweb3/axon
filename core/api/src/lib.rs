pub mod adapter;
pub mod graphql;
pub mod jsonrpc;

pub use adapter::DefaultAPIAdapter;

use std::error::Error;

use protocol::{Display, ProtocolError, ProtocolErrorKind};

#[derive(Debug, Display)]
pub enum APIError {
    #[display(fmt = "adapter error {:?}", _0)]
    Adapter(String),

    #[display(fmt = "http server error {:?}", _0)]
    HttpServer(String),

    #[display(fmt = "web socket server error {:?}", _0)]
    WebSocketServer(String),

    #[display(fmt = "storage error {:?}", _0)]
    Storage(String),
}

impl Error for APIError {}

impl From<APIError> for ProtocolError {
    fn from(error: APIError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::API, Box::new(error))
    }
}
