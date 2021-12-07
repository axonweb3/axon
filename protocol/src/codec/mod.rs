pub mod error;
pub mod transaction;

use rlp::{Decodable, Encodable};

use crate::{types::Bytes, ProtocolResult};

pub trait ProtocolCodec: Sized + Send {
    fn encode(&self) -> ProtocolResult<Bytes>;

    fn decode(bytes: Bytes) -> ProtocolResult<Self>;
}

impl<T: Encodable + Decodable + Send> ProtocolCodec for T {
    fn encode(&self) -> ProtocolResult<Bytes> {
        Ok(rlp::encode(self).freeze())
    }

    fn decode(bytes: Bytes) -> ProtocolResult<Self> {
        rlp::decode(bytes.as_ref()).map_err(|e| error::CodecError::Rlp(e.to_string()).into())
    }
}
