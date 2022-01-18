use reqwest::Client;

use ckb_jsonrpc_types::{
    BlockNumber, BlockView, HeaderView, OutputsValidator, Transaction, TransactionView,
    TransactionWithStatus,
};
use ckb_types::{packed, prelude::*, H160, H256};
use common_crypto::{HashValue, PrivateKey, Secp256k1RecoverablePrivateKey, Signature};
use futures::Future;
use protocol::{codec::hex_encode, types::Bytes};
use serde::{Deserialize, Serialize};
use std::{
    cmp, io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

#[allow(clippy::upper_case_acronyms)]
enum Target {
    CKB,
    Mercury,
}

macro_rules! jsonrpc {
    ($method:expr, $id:expr, $self:ident, $return:ty$(, $params:ident$(,)?)*) => {{
        let data = format!(
            r#"{{"id": {}, "jsonrpc": "2.0", "method": "{}", "params": {}}}"#,
            $self.id.load(Ordering::Relaxed),
            $method,
            serde_json::to_value(($($params,)*)).unwrap()
        );
        $self.id.fetch_add(1, Ordering::Relaxed);

        let req_json: serde_json::Value = serde_json::from_str(&data).unwrap();

        let url = match $id {
            Target::CKB => $self.ckb_uri.clone(),
            Target::Mercury => $self.mercury_uri.clone(),
        };
        let c = $self.raw.post(url).json(&req_json);
        async {
            let resp = c
                .send()
                .await
                .map_err::<io::Error, _>(|_| io::ErrorKind::ConnectionAborted.into())?;
            let output = resp
                .json::<jsonrpc_core::response::Output>()
                .await
                .map_err::<io::Error, _>(|_| io::ErrorKind::InvalidData.into())?;

            match output {
                jsonrpc_core::response::Output::Success(success) => {
                    Ok(serde_json::from_value::<$return>(success.result).unwrap())
                }
                jsonrpc_core::response::Output::Failure(e) => {
                    Err(io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", e)))
                }
            }
        }
    }}
}

#[derive(Clone)]
pub struct RpcClient {
    raw:         Client,
    ckb_uri:     reqwest::Url,
    mercury_uri: reqwest::Url,
    id:          Arc<AtomicU64>,
}

impl RpcClient {
    pub fn new(ckb_uri: &str, mercury_uri: &str) -> Self {
        let ckb_uri =
            reqwest::Url::parse(ckb_uri).expect("ckb uri, e.g. \"http://127.0.0.1:8114\"");
        let mercury_uri =
            reqwest::Url::parse(mercury_uri).expect("ckb uri, e.g. \"http://127.0.0.1:8116\"");
        RpcClient {
            raw: Client::new(),
            ckb_uri,
            mercury_uri,
            id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn get_block_by_number(
        &mut self,
        number: BlockNumber,
    ) -> impl Future<Output = io::Result<BlockView>> {
        jsonrpc!("get_block_by_number", Target::CKB, self, BlockView, number)
    }

    pub fn get_tip_header(&self) -> impl Future<Output = io::Result<HeaderView>> {
        jsonrpc!("get_tip_header", Target::CKB, self, HeaderView)
    }

    pub fn get_transaction(
        &self,
        hash: &H256,
    ) -> impl Future<Output = io::Result<Option<TransactionWithStatus>>> {
        jsonrpc!(
            "get_transaction",
            Target::CKB,
            self,
            Option<TransactionWithStatus>,
            hash
        )
    }

    pub fn send_transaction(
        &self,
        tx: &Transaction,
        outputs_validator: Option<OutputsValidator>,
    ) -> impl Future<Output = io::Result<H256>> {
        jsonrpc!(
            "send_transaction",
            Target::CKB,
            self,
            H256,
            tx,
            outputs_validator
        )
    }

    pub fn build_cross_chain_transfer_transaction(
        &self,
        paylod: CrossChainTransferPayload,
    ) -> impl Future<Output = io::Result<TransactionCompletionResponse>> {
        jsonrpc!(
            "build_cross_chain_transfer_transaction",
            Target::Mercury,
            self,
            TransactionCompletionResponse,
            paylod
        )
    }

    pub fn build_submit_checkpoint_transaction(
        &self,
        paylod: SubmitCheckpointPayload,
    ) -> impl Future<Output = io::Result<TransactionCompletionResponse>> {
        jsonrpc!(
            "build_submit_checkpoint_transaction",
            Target::Mercury,
            self,
            TransactionCompletionResponse,
            paylod
        )
    }
}

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
    #[allow(clippy::identity_op)]
    pub fn sign(self, key: &Secp256k1RecoverablePrivateKey) -> Transaction {
        let tx: packed::Transaction = Into::<packed::Transaction>::into(self.tx_view.inner);

        let mut groups = Vec::with_capacity(self.signature_actions.len());
        for action in self.signature_actions {
            groups.push(ScriptGroup {
                original_witness: tx
                    .witnesses()
                    .get(action.signature_location.index)
                    .unwrap()
                    .unpack(),
                action,
            })
        }

        let mut new_witnesses_index = Vec::new();
        let tx_hash = tx.calc_tx_hash();

        for group in groups {
            let group_witnesses = group.group_witnesses(&tx);

            let mut blake2b = ckb_hash::new_blake2b();
            // let mut message = Vec::new();

            blake2b.update(tx_hash.as_slice());
            println!("tx hash: {:?}", hex_encode(tx_hash.as_slice()));
            blake2b.update(
                &(tx.witnesses()
                    .get_unchecked(group.action.signature_location.index)
                    .raw_data()
                    .len() as u64)
                    .to_le_bytes(),
            );

            println!(
                "tx witness len: {:?}",
                tx.witnesses()
                    .get_unchecked(group.action.signature_location.index)
                    .raw_data()
                    .len() as u64
            );

            blake2b.update(
                &tx.witnesses()
                    .get_unchecked(group.action.signature_location.index)
                    .raw_data(),
            );

            println!(
                "tx witness: {:?}",
                hex_encode(
                    &tx.witnesses()
                        .get_unchecked(group.action.signature_location.index)
                        .raw_data()
                )
            );

            for witness in group_witnesses.iter().skip(1) {
                blake2b.update(&(witness.len() as u64).to_le_bytes());
                blake2b.update(witness.as_ref());
            }

            let mut hash = [0u8; 32];
            blake2b.finalize(&mut hash);
            println!("{:?}", hex_encode(hash));

            let mut signature = key
                .sign_message(&HashValue::from_bytes_unchecked(hash))
                .to_bytes();

            // let mut new_witness: packed::Bytes = group.original_witness.pack();

            println!("{:?}", group.original_witness.to_vec());
            let witness = ckb_types::packed::WitnessArgs::new_unchecked(group.original_witness)
                .as_builder()
                .lock(Some(signature).pack())
                .build()
                .as_bytes();

            new_witnesses_index.push((group.action.signature_location.index, witness));
        }

        let witnesses = tx.witnesses();

        let mut new_witnesses = Vec::new();
        for i in 0..witnesses.item_count() {
            new_witnesses.push(witnesses.get_unchecked(i));
        }

        for (idx, witness) in new_witnesses_index {
            new_witnesses[idx] = witness.pack();
        }

        let builder = witnesses.as_builder().set(new_witnesses).build();

        tx.as_builder().witnesses(builder).build().into()
    }
}

struct ScriptGroup {
    action:           SignatureAction,
    original_witness: Bytes,
}

impl ScriptGroup {
    #[allow(clippy::identity_op)]
    fn group_witnesses(&self, tx: &packed::Transaction) -> Vec<Bytes> {
        let mut group_witnesses = Vec::with_capacity(self.action.other_indexes_in_group.len() + 1);
        group_witnesses.push(
            tx.witnesses()
                .get_unchecked(self.action.signature_location.index)
                .raw_data(),
        );
        for idx in self.action.other_indexes_in_group.iter() {
            group_witnesses.push(tx.witnesses().get_unchecked(*idx).raw_data());
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

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SubmitCheckpointPayload {
    pub node_id:              Identity,
    pub admin_id:             Identity,
    pub period_number:        u64,
    pub checkpoint:           Bytes,
    pub selection_lock_hash:  H256,
    pub checkpoint_type_hash: H256,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Identity {
    pub flag:    u8,
    pub content: Bytes,
}

impl Identity {
    pub fn new(flag: u8, content: Vec<u8>) -> Self {
        Self {
            flag,
            content: hex_encode(content).into_bytes().into(),
        }
    }
}
