use std::sync::Arc;

use tendermint_rpc::endpoint::broadcast::{tx_async, tx_commit, tx_sync};
use tendermint_rpc::endpoint::{
    abci_info, abci_query, block, block_results, block_search, blockchain, commit,
    consensus_params, consensus_state, evidence, net_info, status, tx, tx_search, validators,
};

use protocol::{async_trait, traits::APIAdapter};

use crate::jsonrpc::{tendermint::TendermintRpcServer, RpcResult};

pub struct TendermintRpcImpl<Adapter> {
    _adapter: Arc<Adapter>,
}

#[async_trait]
impl<Adapter: APIAdapter + 'static> TendermintRpcServer for TendermintRpcImpl<Adapter> {
    async fn abci_info(&self) -> RpcResult<abci_info::Response> {
        todo!()
    }

    async fn abci_query(&self, _req: abci_query::Request) -> RpcResult<abci_query::Response> {
        todo!()
    }

    async fn block(&self, _req: block::Request) -> RpcResult<block::Response> {
        todo!()
    }

    async fn block_results(
        &self,
        _req: block_results::Request,
    ) -> RpcResult<block_results::Response> {
        todo!()
    }

    async fn block_search(&self, _req: block_search::Request) -> RpcResult<block_search::Response> {
        todo!()
    }

    async fn blockchain(&self, _req: blockchain::Request) -> RpcResult<blockchain::Response> {
        todo!()
    }

    async fn broadcast_tx_async(&self, _req: tx_async::Request) -> RpcResult<tx_async::Response> {
        todo!()
    }

    async fn broadcast_tx_sync(&self, _req: tx_sync::Request) -> RpcResult<tx_sync::Response> {
        todo!()
    }

    async fn broadcast_tx_commit(
        &self,
        _req: tx_commit::Request,
    ) -> RpcResult<tx_commit::Response> {
        todo!()
    }

    async fn commit(&self, _req: commit::Request) -> RpcResult<commit::Response> {
        todo!()
    }

    async fn consensus_params(
        &self,
        _req: consensus_params::Request,
    ) -> RpcResult<consensus_params::Response> {
        todo!()
    }

    async fn consensus_state(&self) -> RpcResult<consensus_state::Response> {
        todo!()
    }

    async fn validators(&self, _req: validators::Request) -> RpcResult<validators::Response> {
        todo!()
    }

    async fn health(&self) -> RpcResult<()> {
        todo!()
    }

    async fn net_info(&self) -> RpcResult<net_info::Response> {
        todo!()
    }

    async fn status(&self) -> RpcResult<status::Response> {
        todo!()
    }

    async fn broadcast_evidence(&self, _req: evidence::Request) -> RpcResult<evidence::Response> {
        todo!()
    }

    async fn tx(&self, _req: tx::Request) -> RpcResult<tx::Response> {
        todo!()
    }

    async fn tx_search(&self, _req: tx_search::Request) -> RpcResult<tx_search::Response> {
        todo!()
    }
}

impl<Adapter: APIAdapter + 'static> TendermintRpcImpl<Adapter> {
    pub fn new(_adapter: Arc<Adapter>) -> Self {
        TendermintRpcImpl { _adapter }
    }
}
