use common_config_parser::types::ConfigApi;
use jsonrpsee::{
    http_server::{HttpServerBuilder, HttpServerHandle},
    proc_macros::rpc,
    types::Error,
};
use protocol::{
    traits::{MemPool, Storage},
    types::{BlockNumber, Bytes, RichBlock, SignedTransaction, H256},
};
use std::{io, sync::Arc};

mod adapter;
mod rpc;

type RpcResult<T> = Result<T, Error>;

#[rpc(server)]
pub trait AxonRpc {
    /// Sends signed transaction, returning its hash.
    #[method(name = "eth_sendRawTransaction")]
    async fn send_raw_transaction(&self, tx: Bytes) -> RpcResult<H256>;
    /// Get transaction by its hash.
    #[method(name = "eth_getTransactionByHash")]
    async fn get_transaction_by_hash(&self, hash: H256) -> RpcResult<SignedTransaction>;
    /// Returns block with given number.
    #[method(name = "eth_getBlockByNumber")]
    async fn block_by_number(
        &self,
        number: BlockNumber,
        ignore: bool,
    ) -> RpcResult<Option<RichBlock>>;
}

pub async fn run_http_server<M, S>(
    config: ConfigApi,
    m: Arc<M>,
    s: Arc<S>,
) -> Result<HttpServerHandle, io::Error>
where
    M: MemPool + 'static,
    S: Storage + 'static,
{
    let server = HttpServerBuilder::new()
        .max_request_body_size(config.max_payload_size as u32)
        .build(config.listening_address)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let module = crate::rpc::RpcImpl::new(crate::adapter::Adapter::new(m, s));

    let handle = server
        .start(module.into_rpc())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(handle)
}
