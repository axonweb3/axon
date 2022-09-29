pub mod r#impl;

pub use r#impl::TendermintRpcImpl;

use jsonrpsee::proc_macros::rpc;
use tendermint_rpc::endpoint::broadcast::{tx_async, tx_commit, tx_sync};
use tendermint_rpc::endpoint::{
    abci_info, abci_query, block, block_results, block_search, blockchain, commit,
    consensus_params, consensus_state, evidence, net_info, status, tx, tx_search, validators,
};

use crate::jsonrpc::RpcResult;

#[rpc(server)]
pub trait TendermintRpc {
    #[method(name = "abci_info")]
    async fn abci_info(&self) -> RpcResult<abci_info::Response>;

    #[method(name = "abci_query")]
    async fn abci_query(&self, req: abci_query::Request) -> RpcResult<abci_query::Response>;

    #[method(name = "block")]
    async fn block(&self, req: block::Request) -> RpcResult<block::Response>;

    #[method(name = "block_results")]
    async fn block_results(
        &self,
        req: block_results::Request,
    ) -> RpcResult<block_results::Response>;

    #[method(name = "block_search")]
    async fn block_search(&self, req: block_search::Request) -> RpcResult<block_search::Response>;

    #[method(name = "blockchain")]
    async fn blockchain(&self, req: blockchain::Request) -> RpcResult<blockchain::Response>;

    #[method(name = "broadcast_tx_async")]
    async fn broadcast_tx_async(&self, req: tx_async::Request) -> RpcResult<tx_async::Response>;

    #[method(name = "broadcast_tx_sync")]
    async fn broadcast_tx_sync(&self, req: tx_sync::Request) -> RpcResult<tx_sync::Response>;

    #[method(name = "broadcast_tx_commit")]
    async fn broadcast_tx_commit(&self, req: tx_commit::Request) -> RpcResult<tx_commit::Response>;

    #[method(name = "commit")]
    async fn commit(&self, req: commit::Request) -> RpcResult<commit::Response>;

    #[method(name = "consensus_params")]
    async fn consensus_params(
        &self,
        req: consensus_params::Request,
    ) -> RpcResult<consensus_params::Response>;

    #[method(name = "consensus_state")]
    async fn consensus_state(&self) -> RpcResult<consensus_state::Response>;

    #[method(name = "validators")]
    async fn validators(&self, req: validators::Request) -> RpcResult<validators::Response>;

    #[method(name = "health")]
    async fn health(&self) -> RpcResult<()>;

    #[method(name = "net_info")]
    async fn net_info(&self) -> RpcResult<net_info::Response>;

    #[method(name = "statue")]
    async fn status(&self) -> RpcResult<status::Response>;

    #[method(name = "broadcast_evidence")]
    async fn broadcast_evidence(&self, req: evidence::Request) -> RpcResult<evidence::Response>;

    #[method(name = "tx")]
    async fn tx(&self, req: tx::Request) -> RpcResult<tx::Response>;

    #[method(name = "tx_search")]
    async fn tx_search(&self, req: tx_search::Request) -> RpcResult<tx_search::Response>;
}
