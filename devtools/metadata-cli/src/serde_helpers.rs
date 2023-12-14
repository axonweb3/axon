use serde::{de::Error, Deserialize};
use serde_with::{DeserializeAs, SerializeAs};

pub struct HexBytes;

impl<T> SerializeAs<T> for HexBytes
where
    T: AsRef<[u8]>,
{
    fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let source = source.as_ref();
        let mut ret = vec![0; 2 + source.len() * 2];
        ret[..2].copy_from_slice(b"0x");
        hex::encode_to_slice(source, &mut ret[2..]).map_err(serde::ser::Error::custom)?;

        serializer.serialize_str(unsafe { std::str::from_utf8_unchecked(&ret) })
    }
}

impl<'de, T> DeserializeAs<'de, T> for HexBytes
where
    T: TryFrom<Vec<u8>>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        hex::decode(s.strip_prefix("0x").unwrap_or(&s))
            .map_err(Error::custom)?
            .try_into()
            .map_err(|_e| {
                Error::custom("failed to convert from vector, incorrect length?".to_string())
            })
    }
}

pub struct HexU32;

impl<'de> DeserializeAs<'de, u32> for HexU32 {
    fn deserialize_as<D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s
            .strip_prefix("0x")
            .ok_or_else(|| Error::custom("expect string that starts with `0x`"))?;
        let v = u32::from_str_radix(s, 16).map_err(Error::custom)?;
        Ok(v)
    }
}

pub struct HexU64;

impl<'de> DeserializeAs<'de, u64> for HexU64 {
    fn deserialize_as<D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s
            .strip_prefix("0x")
            .ok_or_else(|| Error::custom("expect string that starts with `0x`"))?;
        let v = u64::from_str_radix(s, 16).map_err(Error::custom)?;
        Ok(v)
    }
}
