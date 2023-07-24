use std::sync::Arc;

use ckb_jsonrpc_types::{CellData, CellInfo, HeaderView as CkbHeaderView, JsonBytes, OutPoint};
use ckb_traits::HeaderProvider;
use ckb_types::core::cell::{CellProvider, CellStatus};
use ckb_types::{packed, prelude::Pack};

use core_executor::DataProvider;
use jsonrpsee::core::Error;
use protocol::traits::{APIAdapter, Context};
use protocol::{async_trait, ckb_blake2b_256, types::H256};

use crate::jsonrpc::{CkbLightClientRpcServer, RpcResult};

#[derive(Clone, Debug)]
pub struct CkbLightClientRpcImpl<Adapter: APIAdapter> {
    adapter: Arc<Adapter>,
}

#[async_trait]
impl<Adapter: APIAdapter + 'static> CkbLightClientRpcServer for CkbLightClientRpcImpl<Adapter> {
    async fn get_block_header_by_hash(&self, hash: H256) -> RpcResult<Option<CkbHeaderView>> {
        let root = self
            .adapter
            .get_image_cell_root(Context::new())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;
        Ok(DataProvider::new(root)
            .get_header(&(hash.0.pack()))
            .map(Into::into))
    }

    async fn get_cell_info(
        &self,
        out_point: OutPoint,
        with_data: bool,
    ) -> RpcResult<Option<CellInfo>> {
        let out_point: packed::OutPoint = out_point.into();
        let root = self
            .adapter
            .get_image_cell_root(Context::new())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        match DataProvider::new(root).cell(&out_point, false) {
            CellStatus::Live(c) => {
                let data = with_data.then_some(c.mem_cell_data).flatten();
                Ok(Some(CellInfo {
                    output: c.cell_output.into(),
                    data:   data.map(|r| CellData {
                        hash:    ckb_types::H256(ckb_blake2b_256(&r)),
                        content: JsonBytes::from_bytes(r),
                    }),
                }))
            }
            _ => Ok(None),
        }
    }
}

impl<Adapter: APIAdapter> CkbLightClientRpcImpl<Adapter> {
    pub fn new(adapter: Arc<Adapter>) -> Self {
        Self { adapter }
    }
}
