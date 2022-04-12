use std::{
    future::Future,
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use ckb_jsonrpc_types::{
    BlockNumber, BlockView, HeaderView, OutputsValidator, Transaction, TransactionWithStatus,
};
use ckb_types::H256;
use futures::FutureExt;
use reqwest::Client;

use protocol::{
    async_trait, tokio,
    traits::{CkbClient, Context, RPC},
    types::{CrossChainTransferPayload, SubmitCheckpointPayload, TransactionCompletionResponse},
    ProtocolError, ProtocolErrorKind,
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
                .map_err::<ProtocolError, _>(|_| ProtocolError::new(ProtocolErrorKind::CkbClient, Box::new(Into::<io::Error>::into(io::ErrorKind::ConnectionAborted))))?;
            let output = resp
                .json::<jsonrpc_core::response::Output>()
                .await
                .map_err::<ProtocolError, _>(|_| ProtocolError::new(ProtocolErrorKind::CkbClient, Box::new(Into::<io::Error>::into(io::ErrorKind::InvalidData))))?;

            match output {
                jsonrpc_core::response::Output::Success(success) => {
                    Ok(serde_json::from_value::<$return>(success.result).unwrap())
                }
                jsonrpc_core::response::Output::Failure(e) => {
                    Err(ProtocolError::new(ProtocolErrorKind::CkbClient, Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", e)))))
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

    fn get_transaction(
        &self,
        hash: &H256,
    ) -> impl Future<Output = Result<Option<TransactionWithStatus>, ProtocolError>> {
        jsonrpc!(
            "get_transaction",
            Target::CKB,
            self,
            Option<TransactionWithStatus>,
            hash
        )
    }
}

#[async_trait]
impl CkbClient for RpcClient {
    fn get_block_by_number(&self, _ctx: Context, number: BlockNumber) -> RPC<BlockView> {
        jsonrpc!("get_block_by_number", Target::CKB, self, BlockView, number).boxed()
    }

    fn get_tip_header(&self, _ctx: Context) -> RPC<HeaderView> {
        jsonrpc!("get_tip_header", Target::CKB, self, HeaderView).boxed()
    }

    fn get_transaction(&self, _ctx: Context, hash: &H256) -> RPC<Option<TransactionWithStatus>> {
        self.get_transaction(hash).boxed()
    }

    fn send_transaction(
        &self,
        _ctx: Context,
        tx: &Transaction,
        outputs_validator: Option<OutputsValidator>,
    ) -> RPC<H256> {
        jsonrpc!(
            "send_transaction",
            Target::CKB,
            self,
            H256,
            tx,
            outputs_validator
        )
        .boxed()
    }

    fn get_txs_by_hashes(
        &self,
        _ctx: Context,
        hashes: Vec<H256>,
    ) -> RPC<Vec<Option<TransactionWithStatus>>> {
        let mut list = Vec::with_capacity(hashes.len());
        let mut res = Vec::with_capacity(hashes.len());
        for hash in hashes {
            let task = self.get_transaction(&hash);
            list.push(tokio::spawn(task));
        }
        async {
            for i in list {
                let r = i.await.unwrap()?;
                res.push(r);
            }

            Ok(res)
        }
        .boxed()
    }

    fn build_cross_chain_transfer_transaction(
        &self,
        _ctx: Context,
        payload: CrossChainTransferPayload,
    ) -> RPC<TransactionCompletionResponse> {
        jsonrpc!(
            "build_cross_chain_transfer_transaction",
            Target::Mercury,
            self,
            TransactionCompletionResponse,
            payload
        )
        .boxed()
    }

    fn build_submit_checkpoint_transaction(
        &self,
        _ctx: Context,
        paylod: SubmitCheckpointPayload,
    ) -> RPC<TransactionCompletionResponse> {
        jsonrpc!(
            "build_submit_checkpoint_transaction",
            Target::Mercury,
            self,
            TransactionCompletionResponse,
            paylod
        )
        .boxed()
    }
}
