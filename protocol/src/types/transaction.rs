pub use ethereum::{
    AccessList, AccessListItem, EIP1559TransactionMessage as TransactionMessage, TransactionAction,
    TransactionRecoveryId, TransactionSignature,
};
use rlp::Encodable;
use serde::{Deserialize, Serialize};

use common_crypto::secp256k1_recover;

use crate::types::{Bytes, BytesMut, Hash, Hasher, Public, TypesError, H160, H256, H520, U256};
use crate::ProtocolResult;

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

impl std::hash::Hash for Transaction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.nonce.hash(state);
        self.max_priority_fee_per_gas.hash(state);
        self.gas_price.hash(state);
        self.gas_limit.hash(state);
        self.value.hash(state);
        self.data.hash(state);
        if let TransactionAction::Call(addr) = self.action {
            addr.hash(state);
        }

        for access in self.access_list.iter() {
            access.address.hash(state);
        }
    }
}

impl Transaction {
    pub fn encode(&self, chain_id: u64, signature: Option<SignatureComponents>) -> BytesMut {
        UnverifiedTransaction {
            unsigned: self.clone(),
            chain_id,
            signature,
            hash: Default::default(),
        }
        .rlp_bytes()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct UnverifiedTransaction {
    pub unsigned:  Transaction,
    pub signature: Option<SignatureComponents>,
    pub chain_id:  u64,
    pub hash:      H256,
}

impl UnverifiedTransaction {
    pub fn calc_hash(mut self) -> Self {
        debug_assert!(self.signature.is_some());
        let hash = Hasher::digest(&self.unsigned.encode(self.chain_id, self.signature.clone()));
        self.hash = hash;
        self
    }

    pub fn check_hash(&self) -> ProtocolResult<()> {
        let calc_hash =
            Hasher::digest(&self.unsigned.encode(self.chain_id, self.signature.clone()));
        if self.hash != calc_hash {
            return Err(TypesError::TxHashMismatch {
                origin: self.hash,
                calc:   calc_hash,
            }
            .into());
        }

        Ok(())
    }

    pub fn signature_hash(&self) -> Hash {
        Hasher::digest(self.unsigned.encode(self.chain_id, None))
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SignatureComponents {
    pub r:          Bytes,
    pub s:          Bytes,
    pub standard_v: u8,
}

impl From<Bytes> for SignatureComponents {
    // assume that all the bytes data are in Ethereum-like format
    fn from(bytes: Bytes) -> Self {
        debug_assert!(bytes.len() == 65);
        SignatureComponents {
            r:          Bytes::from(bytes[0..32].to_vec()),
            s:          Bytes::from(bytes[32..64].to_vec()),
            standard_v: bytes[64],
        }
    }
}

impl From<SignatureComponents> for Bytes {
    fn from(sc: SignatureComponents) -> Self {
        let mut bytes = BytesMut::from(sc.r.as_ref());
        bytes.extend_from_slice(sc.s.as_ref());
        bytes.extend_from_slice(&[sc.standard_v]);
        bytes.freeze()
    }
}

impl SignatureComponents {
    pub fn as_bytes(&self) -> Bytes {
        self.clone().into()
    }

    pub fn is_eth_sig(&self) -> bool {
        self.standard_v <= 1
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SignedTransaction {
    pub transaction: UnverifiedTransaction,
    pub sender:      H160,
    pub public:      Option<Public>,
}

impl TryFrom<UnverifiedTransaction> for SignedTransaction {
    type Error = TypesError;

    fn try_from(utx: UnverifiedTransaction) -> Result<Self, Self::Error> {
        if utx.signature.is_none() {
            return Err(TypesError::Unsigned);
        }

        let hash = utx.signature_hash();
        let public = Public::from_slice(
            &secp256k1_recover(
                hash.as_bytes(),
                utx.signature.as_ref().unwrap().as_bytes().as_ref(),
            )?
            .serialize_uncompressed()[1..65],
        );

        Ok(SignedTransaction {
            transaction: utx.calc_hash(),
            sender:      public_to_address(&public),
            public:      Some(public),
        })
    }
}

impl SignedTransaction {
    pub fn get_to(&self) -> Option<H160> {
        if let TransactionAction::Call(to) = self.transaction.unsigned.action {
            Some(to)
        } else {
            None
        }
    }
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
