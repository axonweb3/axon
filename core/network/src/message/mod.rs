// pub mod serde;
// pub mod serde_multi;

use std::collections::HashMap;

// use common_apm::muta_apm::rustracing_jaeger::span::TraceId;
use prost::Message;

use protocol::types::Bytes;

use crate::endpoint::Endpoint;
use crate::error::{ErrorKind, NetworkError};

#[derive(Default)]
pub struct Headers(HashMap<String, Vec<u8>>);

// impl Headers {
//     pub fn set_trace_id(&mut self, id: TraceId) {
//         self.0
//             .insert("trace_id".to_owned(), id.to_string().into_bytes());
//     }
//
//     pub fn set_span_id(&mut self, id: u64) {
//         self.0
//             .insert("span_id".to_owned(), id.to_be_bytes().to_vec());
//     }
// }

#[derive(Message)]
pub struct NetworkMessage {
    #[prost(map = "string, bytes", tag = "1")]
    pub headers: HashMap<String, Vec<u8>>,

    #[prost(string, tag = "2")]
    pub url: String,

    #[prost(bytes, tag = "3")]
    pub content: Vec<u8>,
}

impl NetworkMessage {
    pub fn new(endpoint: Endpoint, content: Bytes, headers: Headers) -> Self {
        NetworkMessage {
            headers: headers.0,
            url:     endpoint.full_url().to_owned(),
            content: content.to_vec(),
        }
    }

    pub fn trace_id(&self) -> Option<u32> {
        // self.headers
        //     .get("trace_id")
        //     .map(|id| {
        //         String::from_utf8(id.to_owned())
        //             .ok()
        //             .map(|s| TraceId::from_str(&s).ok())
        //             .flatten()
        //     })
        //     .flatten()
        None
    }

    pub fn span_id(&self) -> Option<u64> {
        self.headers.get("span_id").map(|id| {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&id[..8]);
            u64::from_be_bytes(buf)
        })
    }

    pub fn encode(self) -> Result<Bytes, NetworkError> {
        let mut buf = Vec::with_capacity(self.encoded_len());

        <Self as Message>::encode(&self, &mut buf)
            .map_err(|e| ErrorKind::BadMessage(Box::new(e)))?;

        Ok(Bytes::from(buf))
    }

    pub fn decode(bytes: Bytes) -> Result<Self, NetworkError> {
        <Self as Message>::decode(bytes).map_err(|e| ErrorKind::BadMessage(Box::new(e)).into())
    }
}
