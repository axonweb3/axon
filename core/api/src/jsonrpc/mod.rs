mod r#impl;
mod types;

use jsonrpsee::{
    http_server::{HttpServerBuilder, HttpServerHandle},
    proc_macros::rpc,
    types::Error,
};

use common_config_parser::types::ConfigApi;
use protocol::traits::{MemPool, Storage};
use protocol::types::{Bytes, SignedTransaction, H160, H256, U256};
use protocol::ProtocolResult;

use crate::jsonrpc::types::{BlockId, Web3Block, Web3CallRequest, Web3EstimateRequst, Web3Receipt};
use crate::{adapter::DefaultAPIAdapter, APIError};

type RpcResult<T> = Result<T, Error>;

#[rpc(server)]
pub trait AxonJsonRpc {
    /// Sends signed transaction, returning its hash.
    #[method(name = "eth_sendRawTransaction")]
    async fn send_raw_transaction(&self, tx: Bytes) -> RpcResult<H256>;

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
    async fn chain_id(&self) -> RpcResult<U256>;

    #[method(name = "eth_estimateGas")]
    async fn estimate_gas(&self, req: Web3EstimateRequst) -> RpcResult<Option<U256>>;

    #[method(name = "net_version")]
    async fn net_version(&self) -> RpcResult<U256>;

    #[method(name = "eth_call")]
    async fn call(&self, w3crequest: Web3CallRequest) -> RpcResult<Option<Vec<u8>>>;

    #[method(name = "eth_getCode")]
    async fn get_code(&self, address: H160, number: Option<u64>) -> RpcResult<Vec<u8>>;

    #[method(name = "eth_getTransactionReceipt")]
    async fn get_transaction_receipt(&self, hash: H256) -> RpcResult<Option<Web3Receipt>>;

    #[method(name = "net_listening")]
    async fn listening(&self) -> RpcResult<bool>;

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
    let server = HttpServerBuilder::new()
        .max_request_body_size(config.max_payload_size as u32)
        .build(config.listening_address)
        .map_err(|e| APIError::HttpServer(e.to_string()))?;

    let handle = server
        .start(r#impl::JsonRpcImpl::new(adapter).into_rpc())
        .map_err(|e| APIError::HttpServer(e.to_string()))?;

    Ok(handle)
}
