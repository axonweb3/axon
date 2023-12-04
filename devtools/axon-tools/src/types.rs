// The structures (such as Block, Header, etc.) defined in this file have
// corresponding definitions in the 'axon-protocol' package. It's crucial to
// maintain consistency between these definitions for the correct functioning of
// the system. Therefore, any changes in the structure definitions in the
// 'axon-protocol' package must be mirrored here. Regular synchronization checks
// are recommended to ensure the definitions in this file align with those in
// the 'axon-protocol' package.
use crate::error::TypesError;
#[cfg(any(feature = "hex", feature = "impl-serde"))]
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use bytes::{Bytes, BytesMut};
use core::cmp::Ordering;
use ethereum_types::{Bloom, H160, H256, U64};

#[cfg(feature = "impl-serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "impl-rlp")]
use rlp_derive::{RlpDecodable, RlpEncodable};

#[cfg(feature = "hex")]
use crate::hex::{hex_decode, hex_encode};
#[cfg(feature = "hex")]
use crate::Error;
#[cfg(feature = "hex")]
use core::str::FromStr;

#[cfg(feature = "hex")]
const HEX_PREFIX: &str = "0x";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "impl-rlp", derive(RlpEncodable, RlpDecodable))]
pub struct Hex(Bytes);

impl Hex {
    pub fn empty() -> Self {
        Hex(Bytes::default())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn encode<T: AsRef<[u8]>>(src: T) -> Self {
        Hex(BytesMut::from(src.as_ref()).freeze())
    }

    #[cfg(feature = "hex")]
    pub fn as_string(&self) -> String {
        String::from(HEX_PREFIX) + &hex_encode(self.0.as_ref())
    }

    #[cfg(feature = "hex")]
    pub fn as_string_trim0x(&self) -> String {
        hex_encode(self.0.as_ref())
    }

    pub fn as_bytes(&self) -> Bytes {
        self.0.clone()
    }

    #[cfg(feature = "hex")]
    fn is_prefixed(s: &str) -> bool {
        s.starts_with(HEX_PREFIX)
    }
}

impl Default for Hex {
    fn default() -> Self {
        Hex(vec![0u8; 8].into())
    }
}

impl AsRef<[u8]> for Hex {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for Hex {
    fn from(bytes: Vec<u8>) -> Self {
        let bytes = Bytes::from(bytes);
        Hex(bytes)
    }
}

#[cfg(feature = "hex")]
impl FromStr for Hex {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !Self::is_prefixed(s) {
            return Err(crate::Error::HexPrefix);
        }

        Ok(Hex(hex_decode(&s[2..])?.into()))
    }
}

impl From<Hex> for Bytes {
    fn from(bytes: Hex) -> Self {
        bytes.0
    }
}

#[cfg(all(feature = "impl-serde", feature = "std"))]
impl Serialize for Hex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        faster_hex::withpfx_lowercase::serialize(&self.0, serializer)
    }
}

#[cfg(all(feature = "impl-serde", feature = "std"))]
impl<'de> Deserialize<'de> for Hex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .and_then(|s| Hex::from_str(&s).map_err(serde::de::Error::custom))
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "impl-serde", derive(Serialize, Deserialize))]
pub enum BlockVersion {
    #[default]
    V0,
}

impl From<BlockVersion> for u8 {
    fn from(value: BlockVersion) -> Self {
        match value {
            BlockVersion::V0 => 0,
        }
    }
}

impl TryFrom<u8> for BlockVersion {
    type Error = TypesError;

    fn try_from(value: u8) -> Result<Self, TypesError> {
        match value {
            0 => Ok(BlockVersion::V0),
            _ => Err(TypesError::InvalidBlockVersion(value)),
        }
    }
}

pub type Hash = H256;
pub type MerkleRoot = Hash;
pub type BlockNumber = u64;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "impl-rlp",
    derive(rlp_derive::RlpEncodable, rlp_derive::RlpDecodable)
)]
#[cfg_attr(feature = "impl-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExtraData {
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(
            serialize_with = "faster_hex::withpfx_lowercase::serialize",
            deserialize_with = "faster_hex::withpfx_lowercase::deserialize"
        )
    )]
    pub inner: Bytes,
}

// A copy of axon_protocol::types::block::Header, must be updated simultaneously
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "impl-rlp",
    derive(rlp_derive::RlpEncodable, rlp_derive::RlpDecodable)
)]
#[cfg_attr(feature = "impl-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub version:                  BlockVersion,
    pub prev_hash:                Hash,
    pub proposer:                 H160,
    pub state_root:               MerkleRoot,
    pub transactions_root:        MerkleRoot,
    pub signed_txs_hash:          Hash,
    pub receipts_root:            MerkleRoot,
    pub log_bloom:                Bloom,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(
            serialize_with = "encode::serialize_uint",
            deserialize_with = "decode::deserialize_hex_u64"
        )
    )]
    pub timestamp:                u64,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(
            serialize_with = "encode::serialize_uint",
            deserialize_with = "decode::deserialize_hex_u64"
        )
    )]
    pub number:                   BlockNumber,
    pub gas_used:                 U64,
    pub gas_limit:                U64,
    /// Extra data for the block header
    /// The first index of extra_data is used to store hardfork information:
    /// `HardforkInfoInner`
    pub extra_data:               Vec<ExtraData>,
    pub base_fee_per_gas:         U64,
    pub proof:                    Proof,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(
            serialize_with = "encode::serialize_uint",
            deserialize_with = "decode::deserialize_hex_u32"
        )
    )]
    pub call_system_script_count: u32,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(
            serialize_with = "encode::serialize_uint",
            deserialize_with = "decode::deserialize_hex_u64"
        )
    )]
    pub chain_id:                 u64,
}

// A copy of axon_protocol::types::block::Block, must be updated simultaneously
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "impl-rlp",
    derive(rlp_derive::RlpEncodable, rlp_derive::RlpDecodable)
)]
#[cfg_attr(feature = "impl-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Block {
    pub header:    Header,
    pub tx_hashes: Vec<H256>,
}

// A copy of axon_protocol::types::block::Proposal, must be updated
// simultaneously
#[cfg(feature = "proof")]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(doc_cfg, doc(cfg(feature = "proof")))]
#[cfg_attr(feature = "impl-serde", derive(serde::Deserialize))]
pub struct Proposal {
    pub version:                  BlockVersion,
    pub prev_hash:                Hash,
    pub proposer:                 H160,
    pub prev_state_root:          MerkleRoot,
    pub transactions_root:        MerkleRoot,
    pub signed_txs_hash:          Hash,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub timestamp:                u64,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub number:                   BlockNumber,
    pub gas_limit:                U64,
    pub extra_data:               Vec<ExtraData>,
    pub base_fee_per_gas:         U64,
    pub proof:                    Proof,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub chain_id:                 u64,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u32")
    )]
    pub call_system_script_count: u32,
    pub tx_hashes:                Vec<Hash>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "impl-rlp",
    derive(rlp_derive::RlpEncodable, rlp_derive::RlpDecodable)
)]
#[cfg_attr(feature = "impl-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Proof {
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(
            serialize_with = "encode::serialize_uint",
            deserialize_with = "decode::deserialize_hex_u64"
        )
    )]
    pub number:     u64,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(
            serialize_with = "encode::serialize_uint",
            deserialize_with = "decode::deserialize_hex_u64"
        )
    )]
    pub round:      u64,
    pub block_hash: Hash,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(
            serialize_with = "faster_hex::withpfx_lowercase::serialize",
            deserialize_with = "faster_hex::withpfx_lowercase::deserialize"
        )
    )]
    pub signature:  Bytes,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(
            serialize_with = "faster_hex::withpfx_lowercase::serialize",
            deserialize_with = "faster_hex::withpfx_lowercase::deserialize"
        )
    )]
    pub bitmap:     Bytes,
}

#[cfg(feature = "proof")]
#[derive(rlp_derive::RlpEncodable, rlp_derive::RlpDecodable, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(doc_cfg, doc(cfg(feature = "proof")))]
#[cfg_attr(feature = "impl-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Validator {
    pub pub_key:        Bytes,
    pub propose_weight: u32,
    pub vote_weight:    u32,
}

#[cfg(feature = "proof")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "proof")))]
impl core::cmp::PartialOrd for Validator {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "proof")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "proof")))]
impl core::cmp::Ord for Validator {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.pub_key.cmp(&other.pub_key)
    }
}

#[cfg(feature = "proof")]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(doc_cfg, doc(cfg(feature = "proof")))]
#[cfg_attr(feature = "impl-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vote {
    pub height:     u64,
    pub round:      u64,
    pub vote_type:  u8,
    pub block_hash: Bytes,
}

#[cfg(test)]
#[cfg(feature = "proof")]
impl Vote {
    fn random() -> Self {
        Self {
            height:     rand::random(),
            round:      rand::random(),
            vote_type:  2,
            block_hash: tests::random_bytes(32),
        }
    }
}

#[derive(Default, Clone, Debug, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "impl-rlp",
    derive(rlp_derive::RlpEncodable, rlp_derive::RlpDecodable)
)]
#[cfg_attr(feature = "impl-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MetadataVersion {
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub start: BlockNumber,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
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

#[derive(Default, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "impl-rlp",
    derive(rlp_derive::RlpEncodable, rlp_derive::RlpDecodable)
)]
#[cfg_attr(
    all(feature = "impl-serde", feature = "std"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Metadata {
    pub version:          MetadataVersion,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub epoch:            u64,
    pub verifier_list:    Vec<ValidatorExtend>,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(skip_deserializing)
    )]
    pub propose_counter:  Vec<ProposeCount>,
    pub consensus_config: ConsensusConfig,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "impl-rlp",
    derive(rlp_derive::RlpEncodable, rlp_derive::RlpDecodable)
)]
#[cfg_attr(feature = "impl-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusConfig {
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub gas_limit:       u64,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub interval:        u64,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub propose_ratio:   u64,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub prevote_ratio:   u64,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub precommit_ratio: u64,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub brake_ratio:     u64,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub tx_num_limit:    u64,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub max_tx_size:     u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "impl-rlp", derive(RlpEncodable, RlpDecodable))]
#[cfg_attr(feature = "impl-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProposeCount {
    pub address: H160,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u64")
    )]
    pub count:   u64,
}

#[derive(Clone, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "impl-rlp",
    derive(rlp_derive::RlpEncodable, rlp_derive::RlpDecodable)
)]
#[cfg_attr(
    all(feature = "impl-serde", feature = "std"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ValidatorExtend {
    pub bls_pub_key:    Hex,
    pub pub_key:        Hex,
    pub address:        H160,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u32")
    )]
    pub propose_weight: u32,
    #[cfg_attr(
        all(feature = "impl-serde", feature = "std"),
        serde(deserialize_with = "decode::deserialize_hex_u32")
    )]
    pub vote_weight:    u32,
}

impl PartialOrd for ValidatorExtend {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValidatorExtend {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pub_key.as_bytes().cmp(&other.pub_key.as_bytes())
    }
}

#[cfg(feature = "std")]
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

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "impl-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodePubKey {
    pub bls_pub_key: Bytes,
    pub pub_key:     Bytes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "impl-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CkbRelatedInfo {
    pub metadata_type_id:     H256,
    pub checkpoint_type_id:   H256,
    pub xudt_args:            H256,
    pub stake_smt_type_id:    H256,
    pub delegate_smt_type_id: H256,
    pub reward_smt_type_id:   H256,
}

#[cfg(all(feature = "impl-serde", feature = "std"))]
mod encode {
    use ethereum_types::U256;
    use serde::ser::Serializer;
    static CHARS: &[u8] = b"0123456789abcdef";

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
}

#[cfg(all(feature = "impl-serde", feature = "std"))]
mod decode {
    use ethereum_types::U256;
    use serde::de::{Deserialize, Deserializer};

    pub fn from_hex(hex: &str) -> Result<Vec<u8>, &'static str> {
        let mut bytes = Vec::with_capacity((hex.len() + 1) / 2);

        let mut start_i = 0;
        if hex.len() % 2 != 0 {
            let byte =
                u8::from_str_radix(&hex[0..1], 16).map_err(|_| "Failed to parse hex string")?;
            bytes.push(byte);
            start_i = 1;
        }

        for i in (start_i..hex.len()).step_by(2) {
            let end_i = if i + 2 > hex.len() { i + 1 } else { i + 2 };
            let byte =
                u8::from_str_radix(&hex[i..end_i], 16).map_err(|_| "Failed to parse hex string")?;
            bytes.push(byte);
        }

        Ok(bytes)
    }

    pub fn deserialize_hex_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "0x0" {
            return Ok(0);
        }

        if s.len() >= 2 && &s[0..2] == "0x" {
            let bytes = from_hex(&s[2..]).map_err(serde::de::Error::custom)?;
            let val = U256::from_big_endian(&bytes);
            Ok(val.low_u32())
        } else {
            Err(serde::de::Error::custom("Invalid format"))
        }
    }

    pub fn deserialize_hex_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "0x0" {
            return Ok(0);
        }

        if s.len() >= 2 && &s[0..2] == "0x" {
            let bytes = from_hex(&s[2..]).map_err(serde::de::Error::custom)?;
            let val = U256::from_big_endian(&bytes);
            Ok(val.low_u64())
        } else {
            Err(serde::de::Error::custom("Invalid format"))
        }
    }

    #[cfg(test)]
    mod tests {
        #[cfg(all(
            feature = "hex",
            feature = "proof",
            feature = "impl-serde",
            feature = "impl-rlp"
        ))]
        #[test]
        fn test_deserialize_hex_u64() {
            use crate::types::MetadataVersion;

            {
                let json_str = r#"{"start": "0x0", "end": "0x7"}"#;
                let my_struct: MetadataVersion = serde_json::from_str(json_str).unwrap();
                assert_eq!(my_struct.start, 0x0);
                assert_eq!(my_struct.end, 0x7);
            }

            {
                let json_str = r#"{"start": "0x12", "end": "0x233"}"#;
                let my_struct: MetadataVersion = serde_json::from_str(json_str).unwrap();
                assert_eq!(my_struct.start, 0x12);
                assert_eq!(my_struct.end, 0x233);
            }

            {
                let json_str = r#"{"start": "0x67fed12", "end": "0x8ddefa09"}"#;
                let my_struct: MetadataVersion = serde_json::from_str(json_str).unwrap();
                assert_eq!(my_struct.start, 0x67fed12);
                assert_eq!(my_struct.end, 0x8ddefa09);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn random_bytes(len: usize) -> Bytes {
        (0..len).map(|_| rand::random()).collect::<Vec<u8>>().into()
    }

    #[test]
    #[cfg(feature = "proof")]
    fn test_vote_codec() {
        let vote = Vote::random();
        let raw = rlp::encode(&vote);
        let decoded: overlord::types::Vote = rlp::decode(&raw).unwrap();
        assert_eq!(vote.height, decoded.height);
        assert_eq!(vote.round, decoded.round);
        assert_eq!(vote.block_hash, decoded.block_hash);
    }
}
