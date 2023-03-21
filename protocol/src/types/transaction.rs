pub use ethereum::{
    AccessList, AccessListItem, EIP1559TransactionMessage as TransactionMessage, TransactionAction,
    TransactionRecoveryId, TransactionSignature,
};
use rlp::{Encodable, RlpStream};
use serde::{Deserialize, Serialize};

use common_crypto::secp256k1_recover;

use crate::types::{Bytes, BytesMut, Hash, Hasher, Public, TypesError, H160, H256, H520, U256};
use crate::ProtocolResult;

pub const GAS_PER_ZERO_BYTE: u64 = 4;
pub const GAS_PER_NONZERO_BYTE: u64 = 68;
pub const GAS_CALL_TRANSACTION: u64 = 21_000;
pub const GAS_CREATE_TRANSACTION: u64 = 32_000;
pub const MAX_PRIORITY_FEE_PER_GAS: u64 = 1_337;
pub const MIN_TRANSACTION_GAS_LIMIT: u64 = 21_000;

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum UnsignedTransaction {
    Legacy(LegacyTransaction),
    Eip2930(Eip2930Transaction),
    Eip1559(Eip1559Transaction),
}

impl UnsignedTransaction {
    pub fn type_(&self) -> u64 {
        match self {
            UnsignedTransaction::Legacy(_) => 0x00,
            UnsignedTransaction::Eip2930(_) => 0x01,
            UnsignedTransaction::Eip1559(_) => 0x02,
        }
    }

    pub fn may_cost(&self) -> U256 {
        if let Some(res) = self.gas_price().checked_mul(*self.gas_limit()) {
            return res
                .checked_add(*self.value())
                .unwrap_or_else(U256::max_value);
        }

        U256::max_value()
    }

    pub fn base_gas(&self) -> u64 {
        let base = match self.action() {
            TransactionAction::Call(_) => GAS_CALL_TRANSACTION,
            TransactionAction::Create => GAS_CREATE_TRANSACTION + GAS_CALL_TRANSACTION,
        };

        base + data_gas_cost(self.data())
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

    pub fn set_data(&mut self, data: Bytes) {
        match self {
            UnsignedTransaction::Legacy(tx) => tx.data = data,
            UnsignedTransaction::Eip2930(tx) => tx.data = data,
            UnsignedTransaction::Eip1559(tx) => tx.data = data,
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
        let hash = self.get_hash();
        self.hash = hash;
        self
    }

    pub fn get_hash(&self) -> H256 {
        Hasher::digest(&self.unsigned.encode(self.chain_id, self.signature.clone()))
    }

    pub fn check_hash(&self) -> ProtocolResult<()> {
        let calc_hash = self.get_hash();
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

/// This is only use for test.
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
    pub const ETHEREUM_TX_LEN: usize = 65;

    pub fn as_bytes(&self) -> Bytes {
        self.clone().into()
    }

    pub fn is_eth_sig(&self) -> bool {
        self.len() == Self::ETHEREUM_TX_LEN
    }

    pub fn add_chain_replay_protection(&self, chain_id: Option<u64>) -> u64 {
        chain_id.map(|i| i * 2 + 35).unwrap_or(27)
    }

    pub fn extract_standard_v(v: u64) -> Option<u8> {
        match v {
            v if v >= 35 => Some(((v - 1) % 2) as u8),
            _ => None,
        }
    }

    pub fn extract_chain_id(v: u64) -> Option<u64> {
        if v >= 35 {
            Some((v - 35) / 2u64)
        } else {
            None
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.r.len() + self.s.len() + 1
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SignedTransaction {
    pub transaction: UnverifiedTransaction,
    pub sender:      H160,
    pub public:      Option<Public>,
}

impl SignedTransaction {
    pub fn from_unverified(
        utx: UnverifiedTransaction,
        sender: Option<H160>,
    ) -> ProtocolResult<Self> {
        if utx.signature.is_none() {
            return Err(TypesError::Unsigned.into());
        }

        let hash = utx.signature_hash(true);

        if utx.signature.as_ref().unwrap().is_eth_sig() {
            let public = Public::from_slice(
                &secp256k1_recover(
                    hash.as_bytes(),
                    utx.signature.as_ref().unwrap().as_bytes().as_ref(),
                )
                .map_err(TypesError::Crypto)?
                .serialize_uncompressed()[1..65],
            );

            return Ok(SignedTransaction {
                transaction: utx.calc_hash(),
                sender:      public_to_address(&public),
                public:      Some(public),
            });
        }

        if sender.is_none() {
            return Err(TypesError::MissingInteroperationSender.into());
        }

        Ok(SignedTransaction {
            transaction: utx.calc_hash(),
            sender:      sender.unwrap(),
            public:      Some(Public::zero()),
        })
    }

    pub fn type_(&self) -> u64 {
        self.transaction.unsigned.type_()
    }

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

pub fn data_gas_cost(data: &[u8]) -> u64 {
    let mut ret = 0u64;

    data.iter().for_each(|b| {
        if b == &0u8 {
            ret += GAS_PER_ZERO_BYTE
        } else {
            ret += GAS_PER_NONZERO_BYTE
        }
    });

    ret
}
