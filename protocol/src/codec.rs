use crate::{types::Bytes, ProtocolResult};

pub trait ProtocolCodec: Sized + Send {
    fn encode(&self) -> ProtocolResult<Bytes>;

    fn decode(bytes: Bytes) -> ProtocolResult<Self>;
}
