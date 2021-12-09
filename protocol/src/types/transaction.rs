pub use ethereum::{
    EIP1559Transaction as Transaction, EIP1559TransactionMessage as TransactionMessage,
    TransactionAction, TransactionRecoveryId, TransactionSignature,
};

use crate::types::{Address, Public, H256, U256};
use common_crypto::Secp256k1Signature;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnverifiedTransaction {
    pub unsigned:  Transaction,
    pub signature: SignatureComponents,
    pub chain_id:  Option<u64>,
    pub hash:      H256,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureComponents {
    pub r:          U256,
    pub s:          U256,
    pub standard_v: u8,
}

impl From<SignatureComponents> for Secp256k1Signature {
    fn from(sc: SignatureComponents) -> Self {
        let mut r_bytes = Vec::new();
        let mut s_bytes = Vec::new();
        sc.r.to_big_endian(&mut r_bytes);
        sc.s.to_big_endian(&mut s_bytes);
        r_bytes.append(&mut s_bytes);
        Secp256k1Signature::try_from(r_bytes.as_ref()).unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedTransaction {
    pub transaction: UnverifiedTransaction,
    pub sender:      Address,
    pub public:      Option<Public>,
}
