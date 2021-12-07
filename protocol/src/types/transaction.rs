pub use ethereum::{
    EIP1559Transaction as Transaction, EIP1559TransactionMessage as TransactionMessage,
    TransactionAction, TransactionRecoveryId, TransactionSignature,
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
    pub transaction: UnverifiedTransaction,
    pub sender:      Address,
    pub public:      Option<Public>,
}
