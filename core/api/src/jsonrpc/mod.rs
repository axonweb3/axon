mod r#impl;
mod types;

use std::io;

use jsonrpsee::http_server::{HttpServerBuilder, HttpServerHandle};
use jsonrpsee::{proc_macros::rpc, types::Error};

use common_config_parser::types::ConfigApi;
use protocol::traits::{MemPool, Storage};
use protocol::types::{BlockNumber, Bytes, RichBlock, SignedTransaction, H160, H256, U256};

use crate::adapter::DefaultAPIAdapter;
use crate::jsonrpc::types::{BlockId, CallRequest, Index};

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
        number: BlockNumber,
        ignore: bool,
    ) -> RpcResult<Option<RichBlock>>;

    #[method(name = "eth_block_number")]
    async fn block_number(&self) -> RpcResult<BlockNumber>;

    #[method(name = "eth_getBalance")]
    async fn get_balance(&self, address: H160, number: Option<BlockId>) -> RpcResult<U256>;

    #[method(name = "eth_chainId")]
    async fn chainid(&self) -> RpcResult<U256>;

    #[method(name = "eth_estimateGas")]
    async fn estimate_gas(&self, req: CallRequest, number: Option<BlockId>) -> RpcResult<U256>;
}

pub async fn run_http_server<M, S, DB>(
    config: ConfigApi,
    adapter: DefaultAPIAdapter<M, S, DB>,
) -> Result<HttpServerHandle, io::Error>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    let server = HttpServerBuilder::new()
        .max_request_body_size(config.max_payload_size as u32)
        .build(config.listening_address)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let module = r#impl::JsonRpcImpl::new(adapter);

    let handle = server
        .start(module.into_rpc())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(handle)
}
