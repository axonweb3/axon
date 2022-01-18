mod r#impl;
mod web3_types;

use std::sync::Arc;

use jsonrpsee::http_server::{HttpServerBuilder, HttpServerHandle};
use jsonrpsee::ws_server::{WsServerBuilder, WsServerHandle};
use jsonrpsee::{core::Error, proc_macros::rpc};

use common_config_parser::types::ConfigApi;
use protocol::traits::APIAdapter;
use protocol::types::{Hex, H160, H256, U256};
use protocol::ProtocolResult;

use crate::jsonrpc::web3_types::{
    BlockId, Web3Block, Web3CallRequest, Web3Filter, Web3Log, Web3Receipt,Web3TransactionOut,
};
use crate::APIError;

type RpcResult<T> = Result<T, Error>;

#[rpc(server)]
pub trait AxonJsonRpc {
    /// Sends signed transaction, returning its hash.
    #[method(name = "eth_sendRawTransaction")]
    async fn send_raw_transaction(&self, tx: String) -> RpcResult<H256>;

    /// Get transaction by its hash.
    #[method(name = "eth_getTransactionByHash")]
    async fn get_transaction_by_hash(&self, hash: H256) -> RpcResult<Option<Web3TransactionOut>>;

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

    #[method(name = "eth_getBlockTransactionCountByNumber")]
    async fn get_transaction_count_by_number(&self, number: BlockId) -> RpcResult<U256>;

    #[method(name = "eth_getBalance")]
    async fn get_balance(&self, address: H160, number: BlockId) -> RpcResult<U256>;

    #[method(name = "eth_call")]
    async fn call(&self, req: Web3CallRequest, number: BlockId) -> RpcResult<Hex>;

    #[method(name = "eth_estimateGas")]
    async fn estimate_gas(&self, req: Web3CallRequest, number: Option<BlockId>) -> RpcResult<U256>;

    #[method(name = "eth_chainId")]
    async fn chain_id(&self) -> RpcResult<U256>;

    #[method(name = "net_version")]
    async fn net_version(&self) -> RpcResult<U256>;

    #[method(name = "eth_getCode")]
    async fn get_code(&self, address: H160, number: BlockId) -> RpcResult<Hex>;

    #[method(name = "eth_getTransactionReceipt")]
    async fn get_transaction_receipt(&self, hash: H256) -> RpcResult<Option<Web3Receipt>>;

    #[method(name = "net_listening")]
    async fn listening(&self) -> RpcResult<bool>;

    #[method(name = "eth_gasPrice")]
    async fn gas_price(&self) -> RpcResult<U256>;

    #[method(name = "eth_getLogs")]
    async fn get_logs(&self, filter: Web3Filter) -> RpcResult<Vec<Web3Log>>;
}

pub async fn run_jsonrpc_server<Adapter: APIAdapter + 'static>(
    config: ConfigApi,
    adapter: Arc<Adapter>,
) -> ProtocolResult<(Option<HttpServerHandle>, Option<WsServerHandle>)> {
    let mut ret = (None, None);

    if let Some(addr) = config.http_listening_address {
        let server = HttpServerBuilder::new()
            .max_request_body_size(config.max_payload_size as u32)
            .build(addr)
            .map_err(|e| APIError::HttpServer(e.to_string()))?;

        ret.0 = Some(
            server
                .start(r#impl::JsonRpcImpl::new(Arc::clone(&adapter)).into_rpc())
                .map_err(|e| APIError::HttpServer(e.to_string()))?,
        );
    }

    if let Some(addr) = config.ws_listening_address {
        let server = WsServerBuilder::new()
            .max_request_body_size(config.max_payload_size as u32)
            .max_connections(config.maxconn as u64)
            .build(addr)
            .await
            .map_err(|e| APIError::WebSocketServer(e.to_string()))?;

        ret.1 = Some(
            server
                .start(r#impl::JsonRpcImpl::new(adapter).into_rpc())
                .map_err(|e| APIError::WebSocketServer(e.to_string()))?,
        )
    }

    Ok(ret)
}
