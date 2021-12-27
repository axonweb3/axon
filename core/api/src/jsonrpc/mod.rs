mod r#impl;
mod types;

use std::time::Instant;
use jsonrpsee::http_server::{HttpServerBuilder, HttpServerHandle, AccessControlBuilder};
use jsonrpsee::{proc_macros::rpc, types::{Error,middleware::Middleware}};
use common_config_parser::types::ConfigApi;
use protocol::traits::{MemPool, Storage};
use protocol::types::{ Bytes, SignedTransaction, H160, H256, U256};
use protocol::ProtocolResult;
use crate::jsonrpc::types::{BlockId, Web3Block, Web3TransactionReceipt, Web3SendTrancationRequest, Web3CallRequest};
use crate::{adapter::DefaultAPIAdapter, APIError};

type RpcResult<T> = Result<T, Error>;

#[derive(Clone)]
struct RpcMiddleware;

impl Middleware for RpcMiddleware {
    type Instant = Instant;

    fn on_request(&self) -> Instant {
        Instant::now()
    }

    fn on_call(&self, _name: &str) {
        println!(" method name: {:?}", _name);
    }
}

#[rpc(server)]
pub trait AxonJsonRpc {
   /// Sends signed transaction, returning its hash.
   #[method(name = "eth_sendRawTransaction")]
   async fn send_raw_transaction(&self, tx: Bytes) -> RpcResult<H256>;

   #[method(name = "eth_sendTransaction")]
   async fn send_transaction(&self, tx: Web3SendTrancationRequest) -> RpcResult<Option<H256>>;

   /// Get transaction by its hash.
   #[method(name = "eth_getTransactionByHash")]
   async fn get_transaction_by_hash(&self, hash: H256) -> RpcResult<SignedTransaction>;

   /// Returns block with given number.
   #[method(name = "eth_getBlockByNumber")]
   async fn get_block_by_number(
       &self,
       number: BlockId,
       show_rich_tx: bool,
   ) -> RpcResult<Option<Web3Block>>;

   #[method(name = "eth_blockNumber")]
   async fn block_number(&self) -> RpcResult<U256>;

   #[method(name = "eth_getTransactionCount")]
   async fn get_transaction_count(&self, address: H160, number: BlockId) -> RpcResult<U256>;

   #[method(name = "eth_getBalance")]
   async fn get_balance(&self, address: H160, number: Option<BlockId>) -> RpcResult<U256>;

   #[method(name = "eth_chainId")]
   async fn chainid(&self) -> RpcResult<U256>;

   #[method(name = "eth_estimateGas")]
   async fn estimate_gas(&self, req: Web3CallRequest) -> RpcResult<Option<u64>>;

   #[method(name = "net_version")]
   async fn net_version(&self) -> RpcResult<U256>;

   #[method(name = "eth_call")]
   async fn call(&self, w3crequest: Web3CallRequest) -> RpcResult<Option<Vec<u8>>>;

   #[method(name = "eth_getCode")]
   async fn get_code(&self, address: H160, number: Option<u64>) -> RpcResult<Vec<u8>>;

   #[method(name = "eth_getTransactionReceipt")]
   async fn get_transaction_receipt(
       &self,
       _hash: H256,
   ) -> RpcResult<Option<Web3TransactionReceipt>>;

   #[method(name = "net_listening")]
   async fn listening(&self) -> RpcResult<bool>;

   #[method(name = "eth_accounts")]
   async fn accounts(&self) -> RpcResult<Option<Vec<String>>>;

   #[method(name = "eth_gasPrice")]
   async fn get_gas_price(&self) -> RpcResult<Option<U256>>;
}

pub fn run_http_server<M, S, DB>(
    config: ConfigApi,
    adapter: DefaultAPIAdapter<M, S, DB>,
) -> ProtocolResult<HttpServerHandle>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{

    let access = AccessControlBuilder::default()
        .allow_all_origins()
        .continue_on_invalid_cors(true)
        .build();
        
    let server = HttpServerBuilder::new()
        .max_request_body_size(config.max_payload_size as u32)
        .set_access_control(access)
        .build(config.listening_address)
        .map_err(|e| APIError::HttpServer(e.to_string()))?;

    let handle = server
        .start(r#impl::JsonRpcImpl::new(adapter).into_rpc())
        .map_err(|e| APIError::HttpServer(e.to_string()))?;

    Ok(handle)
}
