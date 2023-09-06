use ckb_jsonrpc_types::{Transaction, TransactionView};
use ckb_types::{packed, prelude::*, H160, H256};
use common_crypto::{HashValue, PrivateKey, Secp256k1RecoverablePrivateKey, Signature};
use serde::{Deserialize, Serialize};

use std::cmp;

use crate::types::{Bytes, Hex};

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CrossChainTransferPayload {
    pub sender:    String,
    pub receiver:  String,
    pub udt_hash:  H256,
    pub amount:    String,
    pub direction: u8,
    pub memo:      H160,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransactionCompletionResponse {
    pub tx_view:           TransactionView,
    pub signature_actions: Vec<SignatureAction>,
}

impl TransactionCompletionResponse {
    pub fn sign(self, key: &Secp256k1RecoverablePrivateKey) -> Transaction {
        let tx: packed::Transaction = Into::<packed::Transaction>::into(self.tx_view.inner);
        let witnesses = tx.witnesses();

        let mut groups = Vec::with_capacity(self.signature_actions.len());
        for action in self.signature_actions {
            groups.push(ScriptGroup {
                original_witness: witnesses
                    .get_unchecked(action.signature_location.index)
                    .unpack(),
                action,
            })
        }

        let mut new_witnesses_index = Vec::new();
        let tx_hash = tx.calc_tx_hash();

        for group in groups {
            let group_witnesses = group.group_witnesses(&witnesses);

            let mut blake2b = ckb_hash::new_blake2b();

            blake2b.update(tx_hash.as_slice());
            blake2b.update(&(group.original_witness.len() as u64).to_le_bytes());
            blake2b.update(&group.original_witness);

            for witness in group_witnesses.iter().skip(1) {
                blake2b.update(&(witness.len() as u64).to_le_bytes());
                blake2b.update(witness.as_ref());
            }

            let mut hash = [0u8; 32];
            blake2b.finalize(&mut hash);

            let signature = key
                .sign_message(&HashValue::from_bytes_unchecked(hash))
                .to_bytes();

            let witness = ckb_types::packed::WitnessArgs::new_unchecked(group.original_witness)
                .as_builder()
                .lock(Some(signature).pack())
                .build()
                .as_bytes();

            new_witnesses_index.push((group.action.signature_location.index, witness));
        }

        let mut new_witnesses = witnesses.as_builder();

        for (idx, witness) in new_witnesses_index {
            new_witnesses.replace(idx, witness.pack());
        }

        let builder = new_witnesses.build();

        tx.as_builder().witnesses(builder).build().into()
    }
}

struct ScriptGroup {
    action:           SignatureAction,
    original_witness: Bytes,
}

impl ScriptGroup {
    fn group_witnesses(&self, witnesses: &packed::BytesVec) -> Vec<Bytes> {
        let mut group_witnesses = Vec::with_capacity(self.action.other_indexes_in_group.len() + 1);
        group_witnesses.push(self.original_witness.clone());
        for idx in self.action.other_indexes_in_group.iter() {
            group_witnesses.push(witnesses.get_unchecked(*idx).raw_data());
        }
        group_witnesses
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignatureAction {
    pub signature_location:     SignatureLocation,
    pub signature_info:         SignatureInfo,
    pub hash_algorithm:         HashAlgorithm,
    pub other_indexes_in_group: Vec<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignatureLocation {
    pub index:  usize, // The index in witensses vector
    pub offset: usize, // The start byte offset in witness encoded bytes
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignatureInfo {
    pub algorithm: SignAlgorithm,
    pub address:   String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SignAlgorithm {
    Secp256k1,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HashAlgorithm {
    Blake2b,
}

impl PartialEq for SignatureAction {
    fn eq(&self, other: &SignatureAction) -> bool {
        self.signature_info.address == other.signature_info.address
            && self.signature_info.algorithm == other.signature_info.algorithm
    }
}

impl Eq for SignatureAction {}

impl PartialOrd for SignatureAction {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SignatureAction {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.signature_location
            .index
            .cmp(&other.signature_location.index)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SubmitCheckpointPayload {
    pub node_id:              Identity,
    pub admin_id:             Identity,
    pub period_number:        u64,
    pub checkpoint:           Bytes,
    pub selection_lock_hash:  H256,
    pub checkpoint_type_hash: H256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub flag:    u8,
    pub content: Hex,
}

impl Identity {
    pub fn new(flag: u8, content: Vec<u8>) -> Self {
        Self {
            flag,
            content: Hex::encode(content),
        }
    }
}
