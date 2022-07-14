pub use ethereum::{
    AccessList, AccessListItem, EIP1559TransactionMessage as TransactionMessage, TransactionAction,
    TransactionRecoveryId, TransactionSignature,
};
use rlp::{Encodable, RlpStream};
use serde::{Deserialize, Serialize};

use common_crypto::secp256k1_recover;

use crate::types::{Bytes, BytesMut, Hash, Hasher, Public, TypesError, H160, H256, H520, U256};
use crate::ProtocolResult;

pub const MAX_PRIORITY_FEE_PER_GAS: u64 = 1_337;
pub const MIN_TRANSACTION_GAS_LIMIT: u64 = 21_000;

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum UnsignedTransaction {
    Legacy(LegacyTransaction),
    Eip2930(Eip2930Transaction),
    Eip1559(Eip1559Transaction),
}

impl UnsignedTransaction {
    pub fn may_cost(&self) -> U256 {
        if let Some(res) = self.gas_price().checked_mul(*self.gas_limit()) {
            return res
                .checked_add(*self.value())
                .unwrap_or_else(U256::max_value);
        }

        U256::max_value()
    }

    pub fn is_legacy(&self) -> bool {
        matches!(self, UnsignedTransaction::Legacy(_))
    }

    pub fn is_eip1559(&self) -> bool {
        matches!(self, UnsignedTransaction::Eip1559(_))
    }

    pub fn data(&self) -> &[u8] {
        match self {
            UnsignedTransaction::Legacy(tx) => tx.data.as_ref(),
            UnsignedTransaction::Eip2930(tx) => tx.data.as_ref(),
            UnsignedTransaction::Eip1559(tx) => tx.data.as_ref(),
        }
    }

    pub fn set_action(&mut self, action: TransactionAction) {
        match self {
            UnsignedTransaction::Legacy(tx) => tx.action = action,
            UnsignedTransaction::Eip2930(tx) => tx.action = action,
            UnsignedTransaction::Eip1559(tx) => tx.action = action,
        }
    }

    pub fn gas_price(&self) -> U256 {
        match self {
            UnsignedTransaction::Legacy(tx) => tx.gas_price,
            UnsignedTransaction::Eip2930(tx) => tx.gas_price,
            UnsignedTransaction::Eip1559(tx) => tx.gas_price.min(tx.max_priority_fee_per_gas),
        }
    }

    pub fn max_priority_fee_per_gas(&self) -> &U256 {
        match self {
            UnsignedTransaction::Legacy(tx) => &tx.gas_price,
            UnsignedTransaction::Eip2930(tx) => &tx.gas_price,
            UnsignedTransaction::Eip1559(tx) => &tx.max_priority_fee_per_gas,
        }
    }

    pub fn get_legacy(&self) -> Option<LegacyTransaction> {
        match self {
            UnsignedTransaction::Legacy(tx) => Some(tx.clone()),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            UnsignedTransaction::Legacy(_) => unreachable!(),
            UnsignedTransaction::Eip2930(_) => 1u8,
            UnsignedTransaction::Eip1559(_) => 2u8,
        }
    }

    pub fn encode(&self, chain_id: u64, signature: Option<SignatureComponents>) -> BytesMut {
        UnverifiedTransaction {
            unsigned: self.clone(),
            chain_id,
            signature,
            hash: Default::default(),
        }
        .rlp_bytes()
    }

    pub fn to(&self) -> Option<H160> {
        match self {
            UnsignedTransaction::Legacy(tx) => tx.get_to(),
            UnsignedTransaction::Eip2930(tx) => tx.get_to(),
            UnsignedTransaction::Eip1559(tx) => tx.get_to(),
        }
    }

    pub fn value(&self) -> &U256 {
        match self {
            UnsignedTransaction::Legacy(tx) => &tx.value,
            UnsignedTransaction::Eip2930(tx) => &tx.value,
            UnsignedTransaction::Eip1559(tx) => &tx.value,
        }
    }

    pub fn gas_limit(&self) -> &U256 {
        match self {
            UnsignedTransaction::Legacy(tx) => &tx.gas_limit,
            UnsignedTransaction::Eip2930(tx) => &tx.gas_limit,
            UnsignedTransaction::Eip1559(tx) => &tx.gas_limit,
        }
    }

    pub fn nonce(&self) -> &U256 {
        match self {
            UnsignedTransaction::Legacy(tx) => &tx.nonce,
            UnsignedTransaction::Eip2930(tx) => &tx.nonce,
            UnsignedTransaction::Eip1559(tx) => &tx.nonce,
        }
    }

    pub fn action(&self) -> &TransactionAction {
        match self {
            UnsignedTransaction::Legacy(tx) => &tx.action,
            UnsignedTransaction::Eip2930(tx) => &tx.action,
            UnsignedTransaction::Eip1559(tx) => &tx.action,
        }
    }

    pub fn access_list(&self) -> AccessList {
        match self {
            UnsignedTransaction::Legacy(_) => Vec::new(),
            UnsignedTransaction::Eip2930(tx) => tx.access_list.clone(),
            UnsignedTransaction::Eip1559(tx) => tx.access_list.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LegacyTransaction {
    pub nonce:     U256,
    pub gas_price: U256,
    pub gas_limit: U256,
    pub action:    TransactionAction,
    pub value:     U256,
    pub data:      Bytes,
}

impl std::hash::Hash for LegacyTransaction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.nonce.hash(state);
        self.gas_price.hash(state);
        self.gas_limit.hash(state);
        self.value.hash(state);
        self.data.hash(state);
        if let TransactionAction::Call(addr) = self.action {
            addr.hash(state);
        }
    }
}

impl LegacyTransaction {
    pub fn get_to(&self) -> Option<H160> {
        match self.action {
            TransactionAction::Call(to) => Some(to),
            TransactionAction::Create => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Eip2930Transaction {
    pub nonce:       U256,
    pub gas_price:   U256,
    pub gas_limit:   U256,
    pub action:      TransactionAction,
    pub value:       U256,
    pub data:        Bytes,
    pub access_list: AccessList,
}

impl std::hash::Hash for Eip2930Transaction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.nonce.hash(state);
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

impl Eip2930Transaction {
    pub fn get_to(&self) -> Option<H160> {
        match self.action {
            TransactionAction::Call(to) => Some(to),
            TransactionAction::Create => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Eip1559Transaction {
    pub nonce:                    U256,
    pub max_priority_fee_per_gas: U256,
    pub gas_price:                U256,
    pub gas_limit:                U256,
    pub action:                   TransactionAction,
    pub value:                    U256,
    pub data:                     Bytes,
    pub access_list:              AccessList,
}

impl std::hash::Hash for Eip1559Transaction {
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

impl Eip1559Transaction {
    pub fn get_to(&self) -> Option<H160> {
        match self.action {
            TransactionAction::Call(to) => Some(to),
            TransactionAction::Create => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct UnverifiedTransaction {
    pub unsigned:  UnsignedTransaction,
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

    /// The `with_chain_id` argument is only used for tests
    pub fn signature_hash(&self, with_chain_id: bool) -> Hash {
        if !with_chain_id {
            if let Some(legacy_tx) = self.unsigned.get_legacy() {
                let mut s = RlpStream::new();
                legacy_tx.rlp_encode(&mut s, None, None);
                return Hasher::digest(s.out());
            }
        }

        Hasher::digest(self.unsigned.encode(self.chain_id, None))
    }

    pub fn recover_public(&self, with_chain_id: bool) -> ProtocolResult<Public> {
        Ok(Public::from_slice(
            &secp256k1_recover(
                self.signature_hash(with_chain_id).as_bytes(),
                self.signature
                    .as_ref()
                    .ok_or(TypesError::MissingSignature)?
                    .as_bytes()
                    .as_ref(),
            )
            .map_err(TypesError::Crypto)?
            .serialize_uncompressed()[1..65],
        ))
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

    pub fn add_chain_replay_protection(&self, chain_id: Option<u64>) -> u64 {
        let id = if let Some(i) = chain_id {
            35 + i * 2
        } else {
            27
        };

        self.standard_v as u64 + id
    }

    pub fn extract_standard_v(v: u64) -> u8 {
        match v {
            v if v == 27 => 0,
            v if v == 28 => 1,
            v if v >= 35 => ((v - 1) % 2) as u8,
            _ => 4,
        }
    }

    pub fn extract_chain_id(v: u64) -> Option<u64> {
        if v >= 35 {
            Some((v - 35) / 2u64)
        } else {
            None
        }
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

        let hash = utx.signature_hash(true);
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
        self.transaction.unsigned.to()
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
