pub mod block;
pub mod error;
pub mod executor;
pub mod receipt;
pub mod transaction;

pub use transaction::truncate_slice;

use ethers_core::utils::parse_checksummed;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use serde::{Deserialize as _, Deserializer, Serializer};

use crate::types::{Address, Bytes, DBBytes, Hex, Key256Bits, TypesError, H160, U256};
use crate::ProtocolResult;

static CHARS: &[u8] = b"0123456789abcdef";

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

pub fn serialize_bytes<S>(val: &Bytes, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    faster_hex::withpfx_lowercase::serialize(val, s)
}

pub fn serialize_uint<S, U>(val: &U, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    U: Into<U256> + Copy,
{
    let val: U256 = (*val).into();
    let mut slice = [0u8; 2 + 64];
    let mut bytes = [0u8; 32];
    val.to_big_endian(&mut bytes);
    let non_zero = bytes.iter().take_while(|b| **b == 0).count();
    let bytes = &bytes[non_zero..];

    if bytes.is_empty() {
        s.serialize_str("0x0")
    } else {
        s.serialize_str(to_hex_raw(&mut slice, bytes, true))
    }
}

pub fn deserialize_256bits_key<'de, D>(deserializer: D) -> Result<Key256Bits, D::Error>
where
    D: Deserializer<'de>,
{
    const LEN: usize = 66;
    let s = String::deserialize(deserializer)?;
    if s.starts_with("0x") || s.starts_with("0X") {
        if s.len() == LEN {
            let slice = &s.as_bytes()[2..];
            let mut v = [0u8; 32];
            faster_hex::hex_decode(slice, &mut v)
                .map(|_| Key256Bits::from(v))
                .map_err(|err| {
                    let msg = format!("failed to parse the 256 bits key since {err}.");
                    serde::de::Error::custom(msg)
                })
        } else {
            let msg = format!(
                "failed to parse the 256 bits key since its length is {} but expect {LEN}.",
                s.len()
            );
            Err(serde::de::Error::custom(msg))
        }
    } else {
        let msg = "failed to parse the 256 bits key since it's not 0x-prefixed.";
        Err(serde::de::Error::custom(msg))
    }
}

pub fn deserialize_address<'de, D>(deserializer: D) -> Result<H160, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_checksummed(&s, None).map_err(|err| {
        let msg = format!("failed to parse the mixed-case checksum address \"{s}\", since {err}.");
        serde::de::Error::custom(msg)
    })
}

fn to_hex_raw<'a>(v: &'a mut [u8], bytes: &[u8], skip_leading_zero: bool) -> &'a str {
    debug_assert!(v.len() > 1 + bytes.len() * 2);

    v[0] = b'0';
    v[1] = b'x';

    let mut idx = 2;
    let first_nibble = bytes[0] >> 4;
    if first_nibble != 0 || !skip_leading_zero {
        v[idx] = CHARS[first_nibble as usize];
        idx += 1;
    }
    v[idx] = CHARS[(bytes[0] & 0xf) as usize];
    idx += 1;

    for &byte in bytes.iter().skip(1) {
        v[idx] = CHARS[(byte >> 4) as usize];
        v[idx + 1] = CHARS[(byte & 0xf) as usize];
        idx += 2;
    }

    // SAFETY: all characters come either from CHARS or "0x", therefore valid UTF8
    unsafe { std::str::from_utf8_unchecked(&v[0..idx]) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random;

    impl Hex {
        fn random() -> Self {
            let data = (0..128).map(|_| random()).collect::<Vec<u8>>();
            Self::from_string(hex_encode(data)).unwrap()
        }
    }

    #[test]
    fn test_hex_codec() {
        let data = (0..128).map(|_| random()).collect::<Vec<u8>>();
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
