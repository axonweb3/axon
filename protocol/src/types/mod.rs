use bytes::Bytes;
use hasher::{Hasher, HasherKeccak};

lazy_static::lazy_static! {
    static ref HASHER_INST: HasherKeccak = HasherKeccak::new();
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    /// Enter an array of bytes to get a 32-bit hash.
    /// Note: sha3 is used for the time being and may be replaced with other
    /// hashing algorithms later.
    pub fn digest<B: AsRef<[u8]>>(bytes: B) -> Self {
        let out = HASHER_INST.digest(bytes.as_ref());
        let mut inner = [0u8; 32];
        inner.copy_from_slice(&out);
        Hash(inner)
    }

    pub fn as_bytes(&self) -> Bytes {
        Bytes::from(self.0.to_vec())
    }
}
