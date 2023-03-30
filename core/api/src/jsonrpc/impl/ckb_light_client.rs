use ckb_jsonrpc_types::{CellData, CellInfo, HeaderView as CkbHeaderView, JsonBytes, OutPoint};
use ckb_traits::HeaderProvider;
use ckb_types::core::cell::{CellProvider, CellStatus};
use ckb_types::{packed, prelude::Pack};

use core_executor::system_contract::DataProvider;

use protocol::{async_trait, ckb_blake2b_256, types::H256};

use crate::jsonrpc::{CkbLightClientRpcServer, RpcResult};

#[derive(Default, Clone, Debug)]
pub struct CkbLightClientRpcImpl;

#[async_trait]
impl CkbLightClientRpcServer for CkbLightClientRpcImpl {
    async fn get_block_header_by_hash(&self, hash: H256) -> RpcResult<Option<CkbHeaderView>> {
        Ok(DataProvider::default()
            .get_header(&(hash.0.pack()))
            .map(Into::into))
    }

    async fn get_cell_info(
        &self,
        out_point: OutPoint,
        with_data: bool,
    ) -> RpcResult<Option<CellInfo>> {
        let out_point: packed::OutPoint = out_point.into();

        match DataProvider::default().cell(&out_point, false) {
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
