use std::{
    future::Future,
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use ckb_jsonrpc_types::{
    BlockNumber, BlockView, HeaderView, JsonBytes, OutputsValidator, Transaction, TransactionView,
    Uint32,
};
use ckb_sdk::rpc::ckb_indexer::{Cell, Order, Pagination, SearchKey};
use ckb_types::H256;
use futures::FutureExt;
use reqwest::{Client, Url};

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
    Indexer,
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
            Target::Indexer => $self.indexer_uri.clone(),
        };
        let c = $self.raw.post(url).json(&req_json);
        async {
            let resp = c
                .send()
                .await
                .map_err::<ProtocolError, _>(|e| ProtocolError::new(ProtocolErrorKind::CkbClient, Box::new(e)))?;
            let output = resp
                .json::<jsonrpc_core::response::Output>()
                .await
                .map_err::<ProtocolError, _>(|e| ProtocolError::new(ProtocolErrorKind::CkbClient, Box::new(e)))?;

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
    ckb_uri:     Url,
    mercury_uri: Url,
    indexer_uri: Url,
    id:          Arc<AtomicU64>,
}

impl RpcClient {
    pub fn new(ckb_uri: &str, mercury_uri: &str, indexer_uri: &str) -> Self {
        let ckb_uri = Url::parse(ckb_uri).expect("ckb uri, e.g. \"http://127.0.0.1:8114\"");
        let mercury_uri = Url::parse(mercury_uri).expect("ckb uri, e.g. \"http://127.0.0.1:8116\"");
        let indexer_uri = Url::parse(indexer_uri).expect("ckb uri, e.g. \"http://127.0.0.1:8116\"");

        RpcClient {
            raw: Client::new(),
            ckb_uri,
            mercury_uri,
            indexer_uri,
            id: Arc::new(AtomicU64::new(0)),
        }
    }

    fn get_transaction(
        &self,
        hash: &H256,
    ) -> impl Future<Output = Result<Option<TransactionView>, ProtocolError>> {
        jsonrpc!(
            "get_transaction",
            Target::CKB,
            self,
            Option<TransactionView>,
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

    fn get_transaction(&self, _ctx: Context, hash: &H256) -> RPC<Option<TransactionView>> {
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
    ) -> RPC<Vec<Option<TransactionView>>> {
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
        payload: SubmitCheckpointPayload,
    ) -> RPC<TransactionCompletionResponse> {
        jsonrpc!(
            "build_submit_checkpoint_transaction",
            Target::Mercury,
            self,
            TransactionCompletionResponse,
            payload
        )
        .boxed()
    }

    fn fetch_live_cells(
        &self,
        _ctx: Context,
        search_key: SearchKey,
        limit: u32,
        cursor: Option<JsonBytes>,
    ) -> RPC<Pagination<Cell>> {
        let order = Order::Asc;
        let limit = Uint32::from(limit);

        jsonrpc!(
            "get_cells",
            Target::Indexer,
            self,
            Pagination<Cell>,
            search_key,
            order,
            limit,
            cursor,
        )
        .boxed()
    }
}
