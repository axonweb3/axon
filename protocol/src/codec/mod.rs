pub mod block;
pub mod error;
pub mod executor;
pub mod transaction;

use rlp::{Decodable, Encodable, Rlp};

use crate::types::{Bytes, DBBytes};
use crate::ProtocolResult;

pub trait ProtocolCodec: Sized + Send {
    fn encode(&self) -> ProtocolResult<Bytes>;

    fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self>;
}

// pub trait ProtocolListCodec: Sized + Send {
//     fn encode_list(&self) -> ProtocolResult<Bytes>;

//     fn decode_list<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self>;
// }

impl<T: Encodable + Decodable + Send> ProtocolCodec for T {
    fn encode(&self) -> ProtocolResult<Bytes> {
        Ok(rlp::encode(self).freeze())
    }

    fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
        rlp::decode(bytes.as_ref()).map_err(|e| error::CodecError::Rlp(e.to_string()).into())
    }
}

// impl<T: Encodable + Decodable + Send> ProtocolListCodec for Vec<T> {
//     fn encode_list(&self) -> ProtocolResult<Bytes> {
//         Ok(rlp::encode_list(&self).freeze())
//     }

//     fn decode_list<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
//         Rlp::new(bytes.as_ref())
//             .as_list()
//             .map_err(|e| error::CodecError::Rlp(e.to_string()).into())
//     }
// }

impl ProtocolCodec for DBBytes {
    fn encode(&self) -> ProtocolResult<Bytes> {
        Ok(self.0.clone())
    }

    fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
        let inner = Bytes::copy_from_slice(bytes.as_ref());
        Ok(Self(inner))
    }
}
