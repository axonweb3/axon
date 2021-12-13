pub use ethereum::{
    EIP1559Transaction as Transaction, EIP1559TransactionMessage as TransactionMessage,
    TransactionAction, TransactionRecoveryId, TransactionSignature,
};

use crate::types::{Address, H160, Bytes, BytesMut, Hasher, Public, H256, H520};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnverifiedTransaction {
    pub unsigned:  Transaction,
    pub signature: SignatureComponents,
    pub chain_id:  u64,
    pub hash:      H256,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureComponents {
    pub r:          H256,
    pub s:          H256,
    pub standard_v: u8,
}

impl From<Bytes> for SignatureComponents {
    fn from(bytes: Bytes) -> Self {
        debug_assert!(bytes.len() == 65);
        SignatureComponents {
            r:          H256::from_slice(&bytes[0..32]),
            s:          H256::from_slice(&bytes[32..64]),
            standard_v: *bytes.as_ref().to_vec().last().unwrap(),
        }
    }
}

impl From<SignatureComponents> for Bytes {
    fn from(sc: SignatureComponents) -> Self {
        let mut bytes = BytesMut::from(sc.r.as_bytes());
        bytes.extend_from_slice(sc.s.as_bytes());
        bytes.extend_from_slice(&[sc.standard_v]);
        bytes.freeze()
    }
}

impl SignatureComponents {
    pub fn as_bytes(&self) -> Bytes {
        self.clone().into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedTransaction {
    pub transaction: UnverifiedTransaction,
    pub sender:      H160,
    pub public:      Public,
}

pub fn public_to_address(public: &Public) -> H160 {
    let hash = Hasher::digest(public);
    let mut ret = H160::zero();
    ret.as_bytes_mut().copy_from_slice(&hash[12..]);
    ret
}

pub fn recover_intact_pub_key(public: &Public) -> H520 {
    let mut inner = vec![4u8];
    inner.extend_from_slice(public.as_bytes());
    H520::from_slice(&inner[0..65])
}
