pub use ethereum_types::{
    Address, Bloom, Public, Secret, Signature, H128, H160, H256, H512, H64, U128, U256, U512,
};

use bytes::Bytes;
use hasher::{Hasher as KeccakHasher, HasherKeccak};

use crate::{types::TypesError, ProtocolResult};

lazy_static::lazy_static! {
    static ref HASHER_INST: HasherKeccak = HasherKeccak::new();
}

pub type Hash = H256;

pub struct Hasher;

impl Hasher {
    pub fn digest<B: AsRef<[u8]>>(bytes: B) -> H256 {
        let hash = HASHER_INST.digest(bytes.as_ref());
        let mut ret = H256::default();
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

pub type MerkleRoot = Hash;
