use bytes::BufMut;
use serde::{Deserialize, Serialize};

use protocol::types::{Bytes, BytesMut};

#[derive(Debug, Deserialize, Serialize)]
pub enum RpcResponse {
    Success(Bytes),
    Error(String),
}

impl RpcResponse {
    pub fn encode(&self) -> Bytes {
        match self {
            RpcResponse::Success(bytes) => {
                let mut b = BytesMut::with_capacity(bytes.len() + 1);
                b.put_u8(0);
                b.put(bytes.as_ref());
                b.freeze()
            }
            RpcResponse::Error(e) => {
                let mut b = BytesMut::with_capacity(e.len() + 1);
                b.put_u8(1);
                b.put(e.as_bytes());
                b.freeze()
            }
        }
    }
}
