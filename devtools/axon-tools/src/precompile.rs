use ckb_types::utilities::{merkle_root, MerkleProof};
use ckb_types::{packed, prelude::*};
use ethers_contract::{EthAbiCodec, EthAbiType};
use rlp::{Encodable, RlpStream};

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

impl Encodable for VerifyProofPayload {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(5);
        s.append(&self.verify_type);
        s.append(&self.transactions_root.as_slice());
        s.append(&self.witnesses_root.as_slice());
        s.append(&self.raw_transactions_root.as_slice());
        s.append(&self.proof);
    }
}

#[derive(EthAbiCodec, EthAbiType, Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    pub indices: Vec<u32>,
    pub lemmas:  Vec<[u8; 32]>,
    pub leaves:  Vec<[u8; 32]>,
}

impl Encodable for Proof {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);
        s.append_list(&self.indices);

        // Append each lemma as a separate item
        s.begin_list(self.lemmas.len());
        for lemma in &self.lemmas {
            s.append(&lemma.as_slice());
        }

        // Append each leaf as a separate item
        s.begin_list(self.leaves.len());
        for leaf in &self.leaves {
            s.append(&leaf.as_slice());
        }
    }
}
/// A standalone function to verify the ckb-merkle-binary-tree proof.
pub fn verify_proof(payload: VerifyProofPayload) -> Result<(), String> {
    // Firstly, verify the transactions_root is consist of the raw_transactions_root
    // and witnesses_root
    let transactions_root: packed::Byte32 = payload.transactions_root.pack();
    let raw_transactions_root: packed::Byte32 = payload.raw_transactions_root.pack();
    let witnesses_root: packed::Byte32 = payload.witnesses_root.pack();

    if merkle_root(&[raw_transactions_root.clone(), witnesses_root.clone()]) != transactions_root {
        return Err(String::from("verify transactions_root fail"));
    }

    // Then, verify the given indices and lemmas can prove the leaves contains in
    // the raw_transactions_root or the witnesses_root.
    // If the verify_type is 0, the leaves should be in the raw_transactions_root,
    // otherwise in the witnesses_root.
    let lemmas = payload
        .proof
        .lemmas
        .iter()
        .map(|l| l.pack())
        .collect::<Vec<_>>();
    let leaves = payload
        .proof
        .leaves
        .iter()
        .map(|l| l.pack())
        .collect::<Vec<_>>();
    let verifying_root = if payload.verify_type == 0 {
        raw_transactions_root
    } else {
        witnesses_root
    };

    if MerkleProof::new(payload.proof.indices, lemmas).verify(&verifying_root, &leaves) {
        return Ok(());
    }

    Err(String::from("verify raw transactions root failed"))
}
