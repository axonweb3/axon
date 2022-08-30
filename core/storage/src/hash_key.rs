use std::str::FromStr;

use protocol::types::{Bytes, Hash, Hasher};
use protocol::{codec::ProtocolCodec, ProtocolResult};

const PREFIX_LEN: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommonPrefix {
    block_height: [u8; PREFIX_LEN], // BigEndian
}

impl CommonPrefix {
    pub fn new(block_height: u64) -> Self {
        CommonPrefix {
            block_height: block_height.to_be_bytes(),
        }
    }

    pub fn len() -> usize {
        PREFIX_LEN
    }

    pub fn height(self) -> u64 {
        u64::from_be_bytes(self.block_height)
    }

    pub fn make_hash_key(self, hash: &Hash) -> [u8; 40] {
        debug_assert!(hash.as_bytes().len() == Hash::len_bytes());

        let mut key = [0u8; 40];
        key[0..8].copy_from_slice(&self.block_height);
        key[8..40].copy_from_slice(&hash.as_bytes()[..32]);

        key
    }
}

impl AsRef<[u8]> for CommonPrefix {
    fn as_ref(&self) -> &[u8] {
        &self.block_height
    }
}

impl From<&[u8]> for CommonPrefix {
    fn from(bytes: &[u8]) -> CommonPrefix {
        debug_assert!(bytes.len() >= PREFIX_LEN);

        let mut h_buf = [0u8; PREFIX_LEN];
        h_buf.copy_from_slice(&bytes[..PREFIX_LEN]);

        CommonPrefix {
            block_height: h_buf,
        }
    }
}

impl ProtocolCodec for CommonPrefix {
    fn encode(&self) -> ProtocolResult<Bytes> {
        Ok(Bytes::copy_from_slice(&self.block_height))
    }

    fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
        Ok(CommonPrefix::from(&bytes.as_ref()[..PREFIX_LEN]))
    }
}

#[derive(Debug, Clone)]
pub struct CommonHashKey {
    prefix: CommonPrefix,
    hash:   Hash,
}

impl CommonHashKey {
    pub fn new(block_height: u64, hash: Hash) -> Self {
        CommonHashKey {
            prefix: CommonPrefix::new(block_height),
            hash,
        }
    }

    pub fn height(&self) -> u64 {
        self.prefix.height()
    }

    pub fn hash(&self) -> &Hash {
        &self.hash
    }
}

impl ProtocolCodec for CommonHashKey {
    fn encode(&self) -> ProtocolResult<Bytes> {
        Ok(Bytes::copy_from_slice(
            &self.prefix.make_hash_key(&self.hash),
        ))
    }

    fn decode<B: AsRef<[u8]>>(bytes: B) -> ProtocolResult<Self> {
        let mut bytes = bytes.as_ref().to_vec();
        debug_assert!(bytes.len() >= CommonPrefix::len());

        let prefix = CommonPrefix::from(&bytes[0..CommonPrefix::len()]);
        let hash = Hash::from_slice(&bytes.split_off(CommonPrefix::len()));

        Ok(CommonHashKey { prefix, hash })
    }
}

impl ToString for CommonHashKey {
    fn to_string(&self) -> String {
        format!("{}:{}", self.prefix.height(), self.hash)
    }
}

impl FromStr for CommonHashKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(':').collect::<Vec<_>>();
        debug_assert!(parts.len() == 2);

        let height = parts[0].parse::<u64>().map_err(|_| ())?;

        let hash = Hasher::digest(parts[1].as_bytes());

        Ok(CommonHashKey::new(height, hash))
    }
}

pub type BlockKey = CommonPrefix;
