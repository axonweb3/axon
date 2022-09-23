mod server;

use std::sync::Arc;

use tendermint::{
    abci::{self, transaction},
    block::Height,
    evidence::Evidence,
    Genesis,
};
use tendermint_rpc::{
    endpoint::{
        abci_info, abci_query, block, block_results, block_search, blockchain, broadcast, commit,
        consensus_params, consensus_state, evidence, status, tx, tx_search, validators,
    },
    query::Query,
};

use protocol::{
    async_trait,
    traits::{Context, Storage, TendermintRpc},
    ProtocolResult,
};

pub use server::run_tm_rpc;

pub struct TendermintRpcAdapter<S: Storage> {
    storage: Arc<S>,
}

impl<S: Storage> TendermintRpcAdapter<S> {
    pub fn new(storage: Arc<S>) -> Self {
        TendermintRpcAdapter { storage }
    }
}

#[async_trait]
impl<S: Storage> TendermintRpc for TendermintRpcAdapter<S> {
    async fn abci_info(&self) -> ProtocolResult<abci_info::AbciInfo> {
        todo!()
    }

    async fn abci_query<V>(
        &self,
        _path: Option<abci::Path>,
        _data: V,
        _height: Option<Height>,
        _prove: bool,
    ) -> ProtocolResult<abci_query::AbciQuery>
    where
        V: Into<Vec<u8>> + Send,
    {
        todo!()
    }

    async fn block<H>(&self, height: Option<H>) -> ProtocolResult<block::Response>
    where
        H: Into<Height> + Send,
    {
        let ctx = Context::new();
        let _block = match height {
            Some(h) => {
                let height: Height = h.into();
                let h = height.value();
                match self.storage.get_block(ctx, h).await {
                    Ok(Some(result)) => Ok(result),
                    _ => todo!(),
                }
            }
            None => self.storage.get_latest_block(ctx).await,
        }?;
        todo!()
    }

    async fn block_results<H>(&self, _height: Option<H>) -> ProtocolResult<block_results::Response>
    where
        H: Into<Height> + Send,
    {
        todo!()
    }

    async fn block_search(
        &self,
        _query: Query,
        _page: u32,
        _per_page: u8,
        _order: tendermint_rpc::Order,
    ) -> ProtocolResult<block_search::Response> {
        todo!()
    }

    async fn blockchain<H>(&self, _min: H, _max: H) -> ProtocolResult<blockchain::Response>
    where
        H: Into<Height> + Send,
    {
        todo!()
    }

    async fn broadcast_tx_async(
        &self,
        _tx: abci::Transaction,
    ) -> ProtocolResult<broadcast::tx_async::Response> {
        todo!()
    }

    async fn broadcast_tx_sync(
        &self,
        _tx: abci::Transaction,
    ) -> ProtocolResult<broadcast::tx_sync::Response> {
        todo!()
    }

    async fn broadcast_tx_commit(
        &self,
        _tx: abci::Transaction,
    ) -> ProtocolResult<broadcast::tx_commit::Response> {
        todo!()
    }

    async fn commit<H>(&self, _height: Option<H>) -> ProtocolResult<commit::Response>
    where
        H: Into<Height> + Send,
    {
        todo!()
    }

    async fn consensus_params<H>(
        &self,
        _height: Option<H>,
    ) -> ProtocolResult<consensus_params::Response>
    where
        H: Into<Height> + Send,
    {
        todo!()
    }

    async fn consensus_state(&self) -> ProtocolResult<consensus_state::Response> {
        todo!()
    }

    async fn validators<H>(
        &self,
        _height: H,
        _paging: tendermint_rpc::Paging,
    ) -> ProtocolResult<validators::Response>
    where
        H: Into<Height> + Send,
    {
        todo!()
    }

    async fn health(&self) -> ProtocolResult<()> {
        todo!()
    }

    async fn genesis<AppState>(&self) -> ProtocolResult<Genesis<AppState>>
    where
        AppState: std::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned + Send,
    {
        todo!()
    }

    async fn net_info(&self) -> ProtocolResult<tendermint_rpc::endpoint::net_info::Response> {
        todo!()
    }

    async fn status(&self) -> ProtocolResult<status::Response> {
        todo!()
    }

    async fn broadcast_evidence(&self, _e: Evidence) -> ProtocolResult<evidence::Response> {
        todo!()
    }

    async fn tx(&self, _hash: transaction::Hash, _prove: bool) -> ProtocolResult<tx::Response> {
        todo!()
    }

    async fn tx_search(
        &self,
        _query: Query,
        _prove: bool,
        _page: u32,
        _per_page: u8,
        _order: tendermint_rpc::Order,
    ) -> ProtocolResult<tx_search::Response> {
        todo!()
    }
}
