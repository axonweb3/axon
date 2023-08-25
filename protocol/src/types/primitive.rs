use std::cmp::Ordering;
use std::{fmt, str::FromStr};

pub use ethereum_types::{
    BigEndianHash, Bloom, Public, Secret, Signature, H128, H160, H256, H512, H520, H64, U128, U256,
    U512, U64,
};
use zeroize::Zeroizing;

use ophelia::{PublicKey, UncompressedPublicKey};
use overlord::DurationConfig;
use rlp_derive::{RlpDecodable, RlpEncodable};
use serde::{de, Deserialize, Serialize};

use common_crypto::Secp256k1PublicKey;
use common_hasher::keccak256;

use crate::codec::{hex_decode, hex_encode, serialize_uint};
use crate::types::{BlockNumber, Bytes, TypesError};
use crate::{ProtocolError, ProtocolResult};

pub type Hash = H256;
pub type MerkleRoot = Hash;

const ADDRESS_LEN: usize = 20;
const HEX_PREFIX: &str = "0x";
const HEX_PREFIX_UPPER: &str = "0X";

pub const NIL_DATA: H256 = H256([
    0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
    0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
]);

// Same value as `hash(rlp(null))`.
// Also be same value as the `root_hash` of an empty `TrieMerkle`.
pub const RLP_NULL: H256 = H256([
    0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
    0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
]);

// Same value as `hash(rlp([]))`.
pub const RLP_EMPTY_LIST: H256 = H256([
    0x1d, 0xcc, 0x4d, 0xe8, 0xde, 0xc7, 0x5d, 0x7a, 0xab, 0x85, 0xb5, 0x67, 0xb6, 0xcc, 0xd4, 0x1a,
    0xd3, 0x12, 0x45, 0x1b, 0x94, 0x8a, 0x74, 0x13, 0xf0, 0xa1, 0x42, 0xfd, 0x40, 0xd4, 0x93, 0x47,
]);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DBBytes(pub Bytes);

impl AsRef<[u8]> for DBBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Vec<u8>> for DBBytes {
    fn from(value: Vec<u8>) -> Self {
        DBBytes(value.into())
    }
}

pub struct Hasher;

impl Hasher {
    pub fn digest<B: AsRef<[u8]>>(bytes: B) -> Hash {
        if bytes.as_ref().is_empty() {
            return NIL_DATA;
        }

        H256(keccak256(bytes))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hex(String);

impl Hex {
    pub fn empty() -> Self {
        Hex(String::from(HEX_PREFIX))
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 2
    }

    pub fn encode<T: AsRef<[u8]>>(src: T) -> Self {
        let mut s = HEX_PREFIX.to_string();
        s.push_str(&hex_encode(src));
        Hex(s)
    }

    pub fn decode(s: String) -> ProtocolResult<Bytes> {
        let s = if Self::is_prefixed(s.as_str()) {
            &s[2..]
        } else {
            s.as_str()
        };

        Ok(Bytes::from(hex_decode(s)?))
    }

    pub fn from_string(s: String) -> ProtocolResult<Self> {
        let s = if Self::is_prefixed(s.as_str()) {
            s
        } else {
            HEX_PREFIX.to_string() + &s
        };

        let _ = hex_decode(&s[2..])?;
        Ok(Hex(s))
    }

    pub fn as_string(&self) -> String {
        self.0.to_owned()
    }

    pub fn as_string_trim0x(&self) -> String {
        (self.0[2..]).to_owned()
    }

    pub fn as_bytes(&self) -> Bytes {
        Bytes::from(hex_decode(&self.0[2..]).expect("impossible, already checked in from_string"))
    }

    fn is_prefixed(s: &str) -> bool {
        s.starts_with(HEX_PREFIX) || s.starts_with(HEX_PREFIX_UPPER)
    }
}

impl Default for Hex {
    fn default() -> Self {
        Hex(String::from("0x0000000000000000"))
    }
}

impl Serialize for Hex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

struct HexVisitor;

impl<'de> de::Visitor<'de> for HexVisitor {
    type Value = Hex;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Expect a hex string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Hex::from_string(v).map_err(|e| de::Error::custom(e.to_string()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Hex::from_string(v.to_owned()).map_err(|e| de::Error::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for Hex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_string(HexVisitor)
    }
}

pub type Key256Bits = Zeroizing<[u8; 32]>;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Address(pub H160);

impl Default for Address {
    fn default() -> Self {
        Address::from_hex("0x0000000000000000000000000000000000000000")
            .expect("Address must consist of 20 bytes")
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_bytes(self.0.as_bytes())
    }
}

// struct AddressVisitor;

// impl<'de> de::Visitor<'de> for AddressVisitor {
//     type Value = Address;

//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         formatter.write_str("Expect a bech32 string")
//     }

//     fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
//     where
//         E: de::Error,
//     {
//         Address::from_str(&v).map_err(|e| de::Error::custom(e.to_string()))
//     }

//     fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
//     where
//         E: de::Error,
//     {
//         Address::from_str(&v).map_err(|e| de::Error::custom(e.to_string()))
//     }
// }

// impl<'de> Deserialize<'de> for Address {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: de::Deserializer<'de>,
//     {
//         deserializer.deserialize_string(AddressVisitor)
//     }
// }

impl Address {
    pub fn from_pubkey_bytes<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
        let compressed_pubkey_len = <Secp256k1PublicKey as PublicKey>::LENGTH;
        let uncompressed_pubkey_len = <Secp256k1PublicKey as UncompressedPublicKey>::LENGTH;

        let slice = bytes.as_ref();
        if slice.len() != compressed_pubkey_len && slice.len() != uncompressed_pubkey_len {
            return Err(TypesError::InvalidPublicKey.into());
        }

        // Drop first byte
        let hash = {
            if slice.len() == compressed_pubkey_len {
                let pubkey = Secp256k1PublicKey::try_from(slice)
                    .map_err(|_| TypesError::InvalidPublicKey)?;
                Hasher::digest(&(pubkey.to_uncompressed_bytes())[1..])
            } else {
                Hasher::digest(&slice[1..])
            }
        };

        Ok(Self::from_hash(hash))
    }

    pub fn from_pubkey(pubkey: &Secp256k1PublicKey) -> Self {
        let hash = Hasher::digest(&(pubkey.to_uncompressed_bytes())[1..]);
        Self::from_hash(hash)
    }

    pub fn from_hash(hash: Hash) -> Self {
        Self(H160::from_slice(&hash.as_bytes()[12..]))
    }

    pub fn from_bytes(bytes: Bytes) -> ProtocolResult<Self> {
        ensure_len(bytes.len(), ADDRESS_LEN)?;
        Ok(Self(H160::from_slice(&bytes.as_ref()[0..20])))
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn from_hex(s: &str) -> ProtocolResult<Self> {
        let s = clean_0x(s)?;
        let bytes = Bytes::from(hex_decode(s)?);
        Self::from_bytes(bytes)
    }

    pub fn eip55(&self) -> String {
        self.to_string()
    }
}

impl FromStr for Address {
    type Err = ProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if checksum(s) != s {
            return Err(TypesError::InvalidCheckSum.into());
        }

        Address::from_hex(&s.to_lowercase())
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let eip55 = checksum(&hex_encode(self.0));
        eip55.fmt(f)?;
        Ok(())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let eip55 = checksum(&hex_encode(self.0));
        eip55.fmt(f)?;
        Ok(())
    }
}

#[derive(
    RlpEncodable, RlpDecodable, Serialize, Deserialize, Default, Clone, Debug, Copy, PartialEq, Eq,
)]
pub struct MetadataVersion {
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub start: BlockNumber,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub end:   BlockNumber,
}

impl MetadataVersion {
    pub fn new(start: BlockNumber, end: BlockNumber) -> Self {
        MetadataVersion { start, end }
    }

    pub fn contains(&self, number: BlockNumber) -> bool {
        self.start <= number && number <= self.end
    }
}

#[derive(
    RlpEncodable, RlpDecodable, Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq,
)]
pub struct Metadata {
    pub version:          MetadataVersion,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub epoch:            u64,
    pub verifier_list:    Vec<ValidatorExtend>,
    #[serde(skip_deserializing)]
    pub propose_counter:  Vec<ProposeCount>,
    pub consensus_config: ConsensusConfig,
}

#[derive(
    RlpEncodable, RlpDecodable, Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq,
)]
pub struct ConsensusConfig {
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub gas_limit:       u64,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub gas_price:       u64,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub interval:        u64,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub propose_ratio:   u64,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub prevote_ratio:   u64,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub precommit_ratio: u64,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub brake_ratio:     u64,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub tx_num_limit:    u64,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub max_tx_size:     u64,
}

impl From<Metadata> for DurationConfig {
    fn from(m: Metadata) -> Self {
        DurationConfig {
            propose_ratio:   m.consensus_config.propose_ratio,
            prevote_ratio:   m.consensus_config.prevote_ratio,
            precommit_ratio: m.consensus_config.precommit_ratio,
            brake_ratio:     m.consensus_config.brake_ratio,
        }
    }
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProposeCount {
    pub address: H160,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub count:   u64,
}

impl ProposeCount {
    pub fn increase(&mut self) {
        self.count += 1;
    }
}

impl From<(H160, u64)> for ProposeCount {
    fn from(value: (H160, u64)) -> Self {
        ProposeCount {
            address: value.0,
            count:   value.1,
        }
    }
}

#[derive(RlpEncodable, RlpDecodable, Clone, Debug, PartialEq, Eq)]
pub struct Validator {
    pub pub_key:        Bytes,
    pub propose_weight: u32,
    pub vote_weight:    u32,
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct ValidatorExtend {
    pub bls_pub_key:    Hex,
    pub pub_key:        Hex,
    pub address:        H160,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub propose_weight: u32,
    #[cfg_attr(feature = "hex-serialize", serde(serialize_with = "serialize_uint"))]
    pub vote_weight:    u32,
}

impl PartialOrd for ValidatorExtend {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.bls_pub_key.cmp(&other.bls_pub_key))
    }
}

impl Ord for ValidatorExtend {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bls_pub_key.cmp(&other.bls_pub_key)
    }
}

impl From<ValidatorExtend> for Validator {
    fn from(ve: ValidatorExtend) -> Self {
        Validator {
            pub_key:        ve.pub_key.as_bytes(),
            propose_weight: ve.propose_weight,
            vote_weight:    ve.vote_weight,
        }
    }
}

impl std::fmt::Debug for ValidatorExtend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let bls_pub_key = self.bls_pub_key.as_string_trim0x();
        let pk = if bls_pub_key.len() > 8 {
            unsafe { bls_pub_key.get_unchecked(0..8) }
        } else {
            bls_pub_key.as_str()
        };

        write!(
            f,
            "bls public key {:?}, public key {:?}, address {:?} propose weight {}, vote weight {}",
            pk, self.pub_key, self.address, self.propose_weight, self.vote_weight
        )
    }
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CkbRelatedInfo {
    pub metadata_type_id:     H256,
    pub checkpoint_type_id:   H256,
    pub xudt_args:            H256,
    pub stake_smt_type_id:    H256,
    pub delegate_smt_type_id: H256,
    pub reward_smt_type_id:   H256,
}

fn ensure_len(real: usize, expect: usize) -> ProtocolResult<()> {
    if real != expect {
        Err(TypesError::LengthMismatch { expect, real }.into())
    } else {
        Ok(())
    }
}

fn clean_0x(s: &str) -> ProtocolResult<&str> {
    if s.starts_with("0x") || s.starts_with("0X") {
        Ok(&s[2..])
    } else {
        Err(TypesError::HexPrefix.into())
    }
}

pub fn checksum(address: &str) -> String {
    let address = address.trim_start_matches("0x").to_lowercase();
    let address_hash = hex_encode(Hasher::digest(address.as_bytes()));

    address
        .char_indices()
        .fold(String::from("0x"), |mut acc, (index, address_char)| {
            // this cannot fail since it's Keccak256 hashed
            let n = u16::from_str_radix(&address_hash[index..index + 1], 16).unwrap();

            if n > 7 {
                // make char uppercase if ith character is 9..f
                acc.push_str(&address_char.to_uppercase().to_string())
            } else {
                // already lowercased
                acc.push(address_char)
            }

            acc
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use common_merkle::TrieMerkle;

    #[test]
    fn test_eip55() {
        let addr = "0x35e70c3f5a794a77efc2ec5ba964bffcc7fd2c0a";
        let eip55 = Address::from_hex(addr).unwrap();
        assert_eq!(
            eip55.to_string(),
            "0x35E70C3F5A794A77Efc2Ec5bA964BFfcC7Fd2C0a"
        );
    }

    #[test]
    fn test_hex_decode() {
        let hex = String::from("0x");
        let res = Hex::from_string(hex.clone()).unwrap();
        assert!(res.is_empty());

        let res = Hex::decode(hex).unwrap();
        assert!(res.is_empty());

        let hex = String::from("123456");
        let _ = Hex::from_string(hex.clone()).unwrap();
        let _ = Hex::decode(hex).unwrap();

        let hex = String::from("0x123f");
        let _ = Hex::from_string(hex.clone()).unwrap();
        let _ = Hex::decode(hex).unwrap();
    }

    #[test]
    fn test_default_values() {
        let bytes = Hex::empty();
        let hash = Hasher::digest(bytes.as_bytes());
        assert_eq!(hash, NIL_DATA);

        let h256_default = H256::default();
        assert!(h256_default.is_zero());

        let null_data: &[u8] = &[];
        let rlp_null_data = rlp::encode(&null_data);
        assert_eq!(&rlp_null_data, &rlp::NULL_RLP[..]);

        let empty_list: Vec<u8> = vec![];
        let rlp_empty_list = rlp::encode_list(&empty_list);
        assert_eq!(&rlp_empty_list, &rlp::EMPTY_LIST_RLP[..]);

        let hash = Hasher::digest(rlp::NULL_RLP);
        assert_eq!(hash, RLP_NULL);

        let hash = Hasher::digest(rlp::EMPTY_LIST_RLP);
        assert_eq!(hash, RLP_EMPTY_LIST);

        let root = TrieMerkle::default().root_hash().unwrap();
        assert_eq!(root, RLP_NULL);
    }

    #[test]
    fn test_metadata_json_serialize() {
        let metadata: Metadata = serde_json::from_slice(
            &fs::read("../devtools/genesis-generator/metadata.json").unwrap(),
        )
        .unwrap();
        let json = serde_json::to_value(metadata).unwrap();

        println!("{:?}", json.to_string());

        assert!(json.get("version").unwrap().is_object());
        assert!(json.get("epoch").unwrap().is_string());
        assert!(json.get("propose_counter").unwrap().is_array());
    }

    #[test]
    fn test_metadata_version() {
        let version_0 = MetadataVersion {
            start: 1,
            end:   100,
        };
        let version_1 = MetadataVersion {
            start: 101,
            end:   200,
        };

        (1..=100).for_each(|n| assert!(version_0.contains(n)));
        (101..=200).for_each(|n| assert!(version_1.contains(n)));
    }
}
