pub mod block;
pub mod error;
pub mod executor;
pub mod receipt;
pub mod transaction;

use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

use crate::types::{Address, Bytes, DBBytes, H160};
use crate::ProtocolResult;

pub trait ProtocolCodec: Sized + Send {
    fn encode(&self) -> ProtocolResult<Bytes>;

    fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self>;
}

impl<T: Encodable + Decodable + Send> ProtocolCodec for T {
    fn encode(&self) -> ProtocolResult<Bytes> {
        Ok(rlp::encode(self).freeze())
    }

    fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
        rlp::decode(bytes.as_ref()).map_err(|e| error::CodecError::Rlp(e.to_string()).into())
    }
}

impl ProtocolCodec for DBBytes {
    fn encode(&self) -> ProtocolResult<Bytes> {
        Ok(self.0.clone())
    }

    fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
        let inner = Bytes::copy_from_slice(bytes.as_ref());
        Ok(Self(inner))
    }
}

impl Encodable for Address {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(1).append(&self.0);
    }
}

impl Decodable for Address {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        let inner: H160 = r.val_at(0)?;
        Ok(Address(inner))
    }
}
