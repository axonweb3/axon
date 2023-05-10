use std::sync::Arc;

use jsonrpsee::core::Error;

use protocol::async_trait;
use protocol::traits::{APIAdapter, Context};
use protocol::types::{Block, Proof};

use crate::jsonrpc::web3_types::BlockId;
use crate::jsonrpc::{AxonRpcServer, RpcResult};

pub struct AxonRpcImpl<Adapter> {
    adapter: Arc<Adapter>,
}

impl<Adapter: APIAdapter> AxonRpcImpl<Adapter> {
    pub fn new(adapter: Arc<Adapter>) -> Self {
        AxonRpcImpl { adapter }
    }
}

#[async_trait]
impl<Adapter: APIAdapter + 'static> AxonRpcServer for AxonRpcImpl<Adapter> {
    async fn get_block_by_id(&self, block_id: BlockId) -> RpcResult<Option<Block>> {
        let ret = match block_id {
            BlockId::Hash(hash) => self.adapter.get_block_by_hash(Context::new(), hash).await,
            BlockId::Num(num) => {
                self.adapter
                    .get_block_by_number(Context::new(), Some(num.as_u64()))
                    .await
            }
            _ => return Err(Error::InvalidRequestId),
        }
        .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(ret)
    }

    async fn get_proof_by_id(&self, block_id: BlockId) -> RpcResult<Option<Proof>> {
        let ret = self
            .get_block_by_id(block_id)
            .await?
            .map(|b| b.header.proof);
        Ok(ret)
    }
}
