pub mod types;

use std::sync::Arc;

use ckb_jsonrpc_types::JsonBytes;
use ckb_sdk::rpc::ckb_indexer::{Cell, Pagination, SearchKey};

use protocol::traits::{CkbClient, Context, TxAssemblerAdapter, RPC};

pub struct IndexerAdapter<C>(Arc<C>);

impl<C> TxAssemblerAdapter for IndexerAdapter<C>
where
    C: CkbClient + 'static,
{
    fn fetch_live_cells(
        &self,
        ctx: Context,
        search_key: SearchKey,
        limit: u32,
        cursor: Option<JsonBytes>,
    ) -> RPC<Pagination<Cell>> {
        self.0.fetch_live_cells(ctx, search_key, limit, cursor)
    }
}

impl<C> IndexerAdapter<C>
where
    C: CkbClient + 'static,
{
    pub fn new(client: Arc<C>) -> Self {
        IndexerAdapter(client)
    }
}
