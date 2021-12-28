pub use ethereum::{
    AccessList, AccessListItem, EIP1559TransactionMessage as TransactionMessage, TransactionAction,
    TransactionRecoveryId, TransactionSignature,
};
use rlp::Encodable;
use serde::{Deserialize, Serialize};

use crate::types::{Bytes, BytesMut, Hash, Hasher, Public, H160, H256, H520, U256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub nonce:                    U256,
    pub max_priority_fee_per_gas: U256,
    pub gas_price:                U256,
    pub gas_limit:                U256,
    pub action:                   TransactionAction,
    pub value:                    U256,
    pub data:                     Bytes,
    pub access_list:              AccessList,
}

impl Transaction {
    pub fn encode(&self, chain_id: u64, signature: Option<SignatureComponents>) -> BytesMut {
        let utx = UnverifiedTransaction {
            unsigned: self.clone(),
            chain_id,
            signature,
            hash: Default::default(),
        };

        utx.rlp_bytes()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UnverifiedTransaction {
    pub unsigned:  Transaction,
    pub signature: Option<SignatureComponents>,
    pub chain_id:  u64,
    pub hash:      H256,
}

impl UnverifiedTransaction {
    pub fn hash(mut self) -> Self {
        let hash = Hasher::digest(&self.rlp_bytes());
        self.hash = hash;
        self
    }

    pub fn check_hash(&self) -> bool {
        Hasher::digest(self.rlp_bytes()) == self.hash
    }

    pub fn signature_hash(&self) -> Hash {
        Hasher::digest(self.unsigned.encode(self.chain_id, None))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SignedTransaction {
    pub transaction: UnverifiedTransaction,
    pub sender:      H160,
    pub public:      Option<Public>,
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
