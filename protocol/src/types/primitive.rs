pub use ethereum_types::{
    Address, Bloom, Public, Secret, Signature, H128, H160, H256, H512, H520, H64, U128, U256, U512,
};
use hasher::{Hasher as KeccakHasher, HasherKeccak};
use overlord::DurationConfig;

use crate::types::{BlockNumber, Bytes, TypesError};
use crate::ProtocolResult;

lazy_static::lazy_static! {
    static ref HASHER_INST: HasherKeccak = HasherKeccak::new();
}

pub type Hash = H256;
pub type MerkleRoot = Hash;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DBBytes(pub Bytes);

impl AsRef<[u8]> for DBBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

pub struct Hasher;

impl Hasher {
    pub fn digest<B: AsRef<[u8]>>(bytes: B) -> Hash {
        let hash = HASHER_INST.digest(bytes.as_ref());
        let mut ret = Hash::default();
        ret.0.copy_from_slice(&hash[0..32]);
        ret
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hex(String);

impl Hex {
    pub fn from_string(s: String) -> ProtocolResult<Self> {
        if (!s.starts_with("0x") && !s.starts_with("0X")) || s.len() < 3 {
            return Err(TypesError::HexPrefix.into());
        }

        hex::decode(&s[2..]).map_err(|error| TypesError::FromHex { error })?;
        Ok(Hex(s))
    }

    pub fn as_string(&self) -> String {
        self.0.to_owned()
    }

    pub fn as_string_trim0x(&self) -> String {
        (&self.0[2..]).to_owned()
    }

    pub fn decode(&self) -> Bytes {
        Bytes::from(hex::decode(&self.0[2..]).expect("impossible, already checked in from_string"))
    }
}

impl Default for Hex {
    fn default() -> Self {
        Hex::from_string("0x1".to_owned()).expect("Hex must start with 0x")
    }
}

#[derive(Default, Clone, Debug, Copy, PartialEq, Eq)]
pub struct MetadataVersion {
    pub start: BlockNumber,
    pub end:   BlockNumber,
}

impl MetadataVersion {
    pub fn new(start: BlockNumber, end: BlockNumber) -> Self {
        MetadataVersion { start, end }
    }

    pub fn contains(&self, number: BlockNumber) -> bool {
        self.start <= number && number < self.end
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub chain_id:        U256,
    pub version:         MetadataVersion,
    pub common_ref:      Hex,
    pub timeout_gap:     u64,
    pub gas_limit:       u64,
    pub gas_price:       u64,
    pub interval:        u64,
    pub verifier_list:   Vec<ValidatorExtend>,
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
    pub tx_num_limit:    u64,
    pub max_tx_size:     u64,
}

impl From<Metadata> for DurationConfig {
    fn from(m: Metadata) -> Self {
        DurationConfig {
            propose_ratio:   m.propose_ratio,
            prevote_ratio:   m.prevote_ratio,
            precommit_ratio: m.precommit_ratio,
            brake_ratio:     m.brake_ratio,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Validator {
    pub pub_key:        Bytes,
    pub propose_weight: u32,
    pub vote_weight:    u32,
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct ValidatorExtend {
    pub bls_pub_key:    Hex,
    pub pub_key:        Hex,
    pub address:        Address,
    pub propose_weight: u32,
    pub vote_weight:    u32,
}

impl From<ValidatorExtend> for Validator {
    fn from(ve: ValidatorExtend) -> Self {
        Validator {
            pub_key:        ve.pub_key.decode(),
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
