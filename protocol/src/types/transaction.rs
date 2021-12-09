pub use ethereum::{
    EIP1559Transaction as Transaction, EIP1559TransactionMessage as TransactionMessage,
    TransactionAction, TransactionRecoveryId, TransactionSignature,
};

use crate::types::{Address, Bytes, Public, H256, U256};

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

impl From<SignatureComponents> for Bytes {
    fn from(sc: SignatureComponents) -> Self {
        let mut r_bytes = Vec::new();
        let mut s_bytes = Vec::new();
        sc.r.to_big_endian(&mut r_bytes);
        sc.s.to_big_endian(&mut s_bytes);
        r_bytes.append(&mut s_bytes);
        r_bytes.push(sc.standard_v);
        Bytes::from(r_bytes)
    }
}

impl SignatureComponents {
    fn as_bytes(&self) -> Bytes {
        self.clone().into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedTransaction {
    pub transaction: UnverifiedTransaction,
    pub sender:      Address,
    pub public:      Option<Public>,
}
