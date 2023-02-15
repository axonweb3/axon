pub mod block;
pub mod crosschain;
pub mod error;
pub mod executor;
pub mod receipt;
pub mod transaction;

use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

use crate::types::{Address, Bytes, DBBytes, Hex, TypesError};
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
        Self::decode(&Rlp::new(bytes.as_ref()))
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
        Ok(Address(r.val_at(0)?))
    }
}

impl Encodable for Hex {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(1).append(&self.as_string_trim0x());
    }
}

impl Decodable for Hex {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        Hex::from_string(r.val_at(0)?).map_err(|_| DecoderError::Custom("hex check"))
    }
}

pub fn hex_encode<T: AsRef<[u8]>>(src: T) -> String {
    faster_hex::hex_string(src.as_ref())
}

pub fn hex_decode(src: &str) -> ProtocolResult<Vec<u8>> {
    if src.is_empty() {
        return Ok(Vec::new());
    }

    let src = if src.starts_with("0x") {
        src.split_at(2).1
    } else {
        src
    };

    let src = src.as_bytes();
    let mut ret = vec![0u8; src.len() / 2];
    faster_hex::hex_decode(src, &mut ret).map_err(TypesError::FromHex)?;

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use getrandom::getrandom;

    impl Hex {
        fn random() -> Self {
            let mut data = [0u8; 128];
            getrandom(&mut data).unwrap();
            Self::from_string(hex_encode(data)).unwrap()
        }
    }

    #[test]
    fn test_hex_codec() {
        let mut data = [0u8; 128];
        getrandom(&mut data).unwrap();
        let data = data.to_vec();

        assert_eq!(hex_encode(&data), hex::encode(data.clone()));
        assert_eq!(
            hex_decode(&hex_encode(&data)).unwrap(),
            hex::decode(hex::encode(data)).unwrap()
        );
        assert!(hex_decode(String::new().as_str()).unwrap().is_empty());
    }

    #[test]
    fn test_hex_rlp() {
        let origin = Hex::random();
        let raw = rlp::encode(&origin);
        assert_eq!(rlp::decode::<Hex>(&raw).unwrap(), origin)
    }
}
