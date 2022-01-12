pub mod block;
pub mod error;
pub mod executor;
pub mod receipt;
pub mod transaction;

use hex_simd::{decode_to_boxed_bytes, encode_to_boxed_str, AsciiCase};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

use crate::types::{Address, Bytes, DBBytes, H160};
use crate::ProtocolResult;

pub trait ProtocolCodec: Sized + Send {
    fn encode(&self) -> ProtocolResult<Bytes>;

    fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self>;
}

impl<T: Encodable + Decodable + Send> ProtocolCodec for T {
    fn encode(&self) -> ProtocolResult<Bytes> {
        Ok(self.rlp_bytes().freeze())
    }

    fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
        Self::decode(&rlp::Rlp::new(bytes.as_ref()))
            .map_err(|e| error::CodecError::Rlp(e.to_string()).into())
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

pub fn hex_encode<T: AsRef<[u8]>>(src: T) -> String {
    encode_to_boxed_str(src.as_ref(), AsciiCase::Lower).into_string()
}

pub fn hex_decode(src: &str) -> ProtocolResult<Vec<u8>> {
    let res = decode_to_boxed_bytes(src.as_bytes())
        .map_err(|error| crate::types::TypesError::FromHex { error })?;
    Ok(res.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_codec() {
        let mut data = [0u8; 128];
        fastrand::shuffle(&mut data);
        let data = data.to_vec();

        assert_eq!(hex_encode(&data), hex::encode(data.clone()));
        assert_eq!(
            hex_decode(&hex_encode(&data)).unwrap(),
            hex::decode(hex::encode(data)).unwrap()
        );
    }
}
