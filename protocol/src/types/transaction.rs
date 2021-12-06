pub use ethereum::{
    EIP2930Transaction as Transaction, EIP2930TransactionMessage as TransactionMessage,
    TransactionRecoveryId, TransactionSignature,
};

use crate::types::{Address, Public, H256, U256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnverifiedTransaction {
    pub unsigned:  Transaction,
    pub signature: SignatureComponents,
    pub chain_id:  Option<u64>,
    pub hash:      H256,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureComponents {
    pub standard_v: u8,
    pub r:          U256,
    pub s:          U256,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedTransaction {
    transaction: UnverifiedTransaction,
    sender:      Address,
    public:      Option<Public>,
}
