use derive_more::Display;
use serde::{Deserialize, Serialize};

use bytes::Bytes;

#[derive(Debug, Deserialize, Serialize, Display)]
#[repr(u8)]
pub enum RpcResponseCode {
    ServerError,
    Other(u8),
}

#[derive(Debug, Deserialize, Serialize, Display)]
#[display(fmt = "rpc err code {} msg {}", code, msg)]
pub struct RpcErrorMessage {
    pub code: RpcResponseCode,
    pub msg:  String,
}

impl std::error::Error for RpcErrorMessage {}

#[derive(Debug, Deserialize, Serialize)]
pub enum RpcResponse {
    Success(Bytes),
    Error(RpcErrorMessage),
}

impl RpcResponse {
    pub fn encode(&self) -> Bytes {
        match self {
            RpcResponse::Success(b) => b.clone(),
            RpcResponse::Error(_) => Bytes::new(),
        }
    }
}
