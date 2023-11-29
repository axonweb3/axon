use ethers_contract::{EthAbiCodec, EthAbiType};

#[derive(EthAbiCodec, EthAbiType, Clone, Debug, PartialEq, Eq)]
pub struct VerifyProofPayload {
    /// If the verify_type is 0, the leaves should be in the
    /// raw_transactions_root, otherwise in the witnesses_root.
    pub verify_type:           u8,
    pub transactions_root:     [u8; 32],
    pub witnesses_root:        [u8; 32],
    pub raw_transactions_root: [u8; 32],
    pub proof:                 Proof,
}

#[derive(EthAbiCodec, EthAbiType, Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    pub indices: Vec<u32>,
    pub lemmas:  Vec<[u8; 32]>,
    pub leaves:  Vec<[u8; 32]>,
}
