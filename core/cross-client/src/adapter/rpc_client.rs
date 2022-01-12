use reqwest::Client;

use ckb_jsonrpc_types::{
    BlockNumber, BlockView, HeaderView, OutputsValidator, Transaction, TransactionWithStatus,
};
use ckb_types::H256;
use futures::Future;
use std::{
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

macro_rules! jsonrpc {
    ($method:expr, $self:ident, $return:ty$(, $params:ident$(,)?)*) => {{
        let data = format!(
            r#"{{"id": {}, "jsonrpc": "2.0", "method": "{}", "params": {}}}"#,
            $self.id.load(Ordering::Relaxed),
            $method,
            serde_json::to_value(($($params,)*)).unwrap()
        );
        $self.id.fetch_add(1, Ordering::Relaxed);

        let req_json: serde_json::Value = serde_json::from_str(&data).unwrap();

        let c = $self.raw.post($self.url.clone()).json(&req_json);
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

pub struct RpcClient {
    raw: Client,
    url: reqwest::Url,
    id:  Arc<AtomicU64>,
}

impl RpcClient {
    pub fn new(uri: &str) -> Self {
        let url = reqwest::Url::parse(uri).expect("ckb uri, e.g. \"http://127.0.0.1:8114\"");
        RpcClient {
            raw: Client::new(),
            url,
            id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn get_block_by_number(
        &mut self,
        number: BlockNumber,
    ) -> impl Future<Output = io::Result<BlockView>> {
        jsonrpc!("get_block_by_number", self, BlockView, number)
    }

    pub fn get_tip_header(&self) -> impl Future<Output = io::Result<HeaderView>> {
        jsonrpc!("get_tip_header", self, HeaderView)
    }

    pub fn get_transaction(
        &self,
        hash: &H256,
    ) -> impl Future<Output = io::Result<Option<TransactionWithStatus>>> {
        jsonrpc!("get_transaction", self, Option<TransactionWithStatus>, hash)
    }

    pub fn send_transaction(
        &self,
        tx: &Transaction,
        outputs_validator: Option<OutputsValidator>,
    ) -> impl Future<Output = io::Result<H256>> {
        jsonrpc!("send_transaction", self, H256, tx, outputs_validator)
    }
}
