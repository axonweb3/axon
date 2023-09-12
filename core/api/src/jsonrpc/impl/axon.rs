use std::{collections::HashMap, sync::Arc};

use jsonrpsee::core::Error;
use strum::IntoEnumIterator;

use common_config_parser::types::spec::HardforkName;
use protocol::async_trait;
use protocol::traits::{APIAdapter, Context};
use protocol::types::{Block, CkbRelatedInfo, Metadata, Proof, Proposal, H256, U256};

use crate::jsonrpc::web3_types::{BlockId, HardforkStatus};
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
            BlockId::Latest => self.adapter.get_block_by_number(Context::new(), None).await,
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

    async fn get_metadata_by_number(&self, block_number: U256) -> RpcResult<Metadata> {
        let ret = self
            .adapter
            .get_metadata_by_number(Context::new(), Some(block_number.as_u64()))
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(ret)
    }

    async fn get_proposal_by_number(&self, block_number: U256) -> RpcResult<Proposal> {
        let block_number = block_number.as_u64();
        let prev_block = self
            .adapter
            .get_block_by_number(Context::new(), Some(block_number - 1))
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .ok_or_else(|| Error::Custom("prev block not found".to_string()))?;
        let block = self
            .adapter
            .get_block_by_number(Context::new(), Some(block_number))
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .ok_or_else(|| Error::Custom("block not found".to_string()))?;

        Ok(Proposal::new_with_state_root(
            &block.header,
            prev_block.header.state_root,
            block.tx_hashes,
        ))
    }

    async fn get_current_metadata(&self) -> RpcResult<Metadata> {
        let ret = self
            .adapter
            .get_metadata_by_number(Context::new(), None)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(ret)
    }

    async fn get_ckb_related_info(&self) -> RpcResult<CkbRelatedInfo> {
        let ret = self
            .adapter
            .get_ckb_related_info(Context::new())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(ret)
    }

    async fn hardfork_infos(&self) -> RpcResult<HashMap<HardforkName, HardforkStatus>> {
        let actived = self
            .adapter
            .hardfork_info(Context::new())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        let proposal = self
            .adapter
            .hardfork_proposal(Context::new())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;
        let actived_latest = actived.inner.last();

        let current_number = self
            .adapter
            .get_block_header_by_number(Default::default(), None)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .unwrap()
            .number;

        let mut hardfork_infos = HashMap::new();
        for hardfork_name in HardforkName::iter() {
            if let Some(p) = proposal.as_ref() {
                if p.flags & H256::from_low_u64_be((hardfork_name as u64).to_be()) != H256::zero() {
                    hardfork_infos.insert(hardfork_name, HardforkStatus::Proposed);
                }
            }
            if let Some(a) = actived_latest {
                if a.flags & H256::from_low_u64_be((hardfork_name as u64).to_be()) != H256::zero() {
                    if a.block_number <= current_number {
                        hardfork_infos.insert(hardfork_name, HardforkStatus::Enabled);
                    } else {
                        hardfork_infos.insert(hardfork_name, HardforkStatus::Activated);
                    }
                }
            }
        }

        Ok(hardfork_infos)
    }
}
