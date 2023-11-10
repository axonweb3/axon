use std::{collections::HashMap, sync::Arc};

use jsonrpsee::{core::RpcResult, types::error::ErrorCode};
use strum::IntoEnumIterator;

use common_config_parser::types::spec::HardforkName;
use protocol::async_trait;
use protocol::traits::{APIAdapter, Context};
use protocol::types::{
    Block, CkbRelatedInfo, HardforkInfoInner, Metadata, Proof, Proposal, H256, U256,
};

use crate::jsonrpc::r#impl::u256_cast_u64;
use crate::jsonrpc::web3_types::{BlockId, HardforkStatus};
use crate::jsonrpc::{error::RpcError, AxonRpcServer};

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
            // The block number is checked when deserialize
            BlockId::Num(num) => {
                self.adapter
                    .get_block_by_number(Context::new(), Some(num.as_u64()))
                    .await
            }
            BlockId::Earliest => self.adapter.get_block_by_number(Context::new(), Some(0)).await,
            BlockId::Latest => self.adapter.get_block_by_number(Context::new(), None).await,
            _ => return Err(ErrorCode::InvalidRequest.into()),
        }
        .map_err(|e| RpcError::Internal(e.to_string()))?;

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
        let block_number = u256_cast_u64(block_number)?;
        let ret = self
            .adapter
            .get_metadata_by_number(Context::new(), Some(block_number))
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?;

        Ok(ret)
    }

    async fn get_proposal_by_number(&self, block_number: U256) -> RpcResult<Proposal> {
        let block_number = block_number.low_u64();

        let prev_state_root = if block_number == 0 {
            H256::default()
        } else {
            self.adapter
                .get_block_by_number(Context::new(), Some(block_number - 1))
                .await
                .map_err(|e| RpcError::Internal(e.to_string()))?
                .ok_or_else(|| RpcError::Internal("prev block not found".to_string()))?
                .header
                .state_root
        };

        let block = self
            .adapter
            .get_block_by_number(Context::new(), Some(block_number))
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?
            .ok_or_else(|| RpcError::Internal("block not found".to_string()))?;

        Ok(Proposal::new_with_state_root(
            &block.header,
            prev_state_root,
            block.tx_hashes,
        ))
    }

    async fn get_current_metadata(&self) -> RpcResult<Metadata> {
        let ret = self
            .adapter
            .get_metadata_by_number(Context::new(), None)
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?;

        Ok(ret)
    }

    async fn get_ckb_related_info(&self) -> RpcResult<CkbRelatedInfo> {
        let ret = self
            .adapter
            .get_ckb_related_info(Context::new())
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?;

        Ok(ret)
    }

    async fn hardfork_infos(&self) -> RpcResult<HashMap<HardforkName, HardforkStatus>> {
        let all_determined = self
            .adapter
            .hardfork_info(Context::new())
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?;

        let proposal = self
            .adapter
            .hardfork_proposal(Context::new())
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?;

        let current_number = self
            .adapter
            .get_block_header_by_number(Default::default(), None)
            .await
            .map_err(|e| RpcError::Internal(e.to_string()))?
            .unwrap()
            .number;

        let (enabled_latest, determined_latest) =
            enabled_and_determined(&all_determined.inner, current_number);

        let mut hardfork_infos = HashMap::new();
        for hardfork_name in HardforkName::iter() {
            if let Some(p) = proposal.as_ref() {
                if p.flags & H256::from_low_u64_be((hardfork_name as u64).to_be()) != H256::zero() {
                    hardfork_infos.insert(hardfork_name, HardforkStatus::Proposed);
                }
            }

            if determined_latest & H256::from_low_u64_be((hardfork_name as u64).to_be())
                != H256::zero()
            {
                hardfork_infos.insert(hardfork_name, HardforkStatus::Determined);
            }

            if enabled_latest & H256::from_low_u64_be((hardfork_name as u64).to_be())
                != H256::zero()
            {
                hardfork_infos.insert(hardfork_name, HardforkStatus::Enabled);
            }
        }

        Ok(hardfork_infos)
    }
}

/// Returns (enabled_flags, determined_flags) in target block height
fn enabled_and_determined(iter: &[HardforkInfoInner], current_number: u64) -> (H256, H256) {
    if iter.len() < 2 {
        match iter.last() {
            Some(latest) => {
                if latest.block_number > current_number {
                    (H256::zero(), latest.flags)
                } else {
                    (latest.flags, H256::zero())
                }
            }
            None => (H256::zero(), H256::zero()),
        }
    } else {
        let (determined, enabled) = {
            let mut determined_latest = None;
            let mut enabled_latest = None;
            for hf in iter.iter().rev() {
                if hf.block_number > current_number {
                    if determined_latest.is_none() {
                        determined_latest = Some(hf.clone());
                    }
                } else {
                    enabled_latest = Some(hf.clone())
                }
                if enabled_latest.is_some() {
                    break;
                }
            }
            (
                determined_latest.map(|i| i.flags).unwrap_or_default(),
                enabled_latest.map(|i| i.flags).unwrap_or_default(),
            )
        };

        (
            enabled,
            if determined != H256::zero() {
                determined ^ enabled
            } else {
                determined
            },
        )
    }
}

#[cfg(test)]
mod test {
    use super::{enabled_and_determined, HardforkInfoInner, H256};

    #[test]
    fn test_select() {
        let v1 = vec![HardforkInfoInner {
            block_number: 0,
            flags:        H256::zero(),
        }];

        let (ve1, vd1) = enabled_and_determined(&v1, 0);

        assert_eq!(ve1, H256::zero());
        assert_eq!(vd1, H256::zero());

        let v2 = vec![HardforkInfoInner {
            block_number: 0,
            flags:        {
                let mut a = [0; 32];
                a[0] = 0b10;
                H256::from(a)
            },
        }];

        let (ve2, vd2) = enabled_and_determined(&v2, 0);

        assert_eq!(ve2, {
            let mut a = [0; 32];
            a[0] = 0b10;
            H256::from(a)
        });
        assert_eq!(vd2, H256::zero());

        let v3 = vec![
            HardforkInfoInner {
                block_number: 0,
                flags:        {
                    let mut a = [0; 32];
                    a[0] = 0b10;
                    H256::from(a)
                },
            },
            HardforkInfoInner {
                block_number: 5,
                flags:        {
                    let mut a = [0; 32];
                    a[0] = 0b11;
                    H256::from(a)
                },
            },
        ];

        let (ve3, vd3) = enabled_and_determined(&v3, 0);

        assert_eq!(ve3, {
            let mut a = [0; 32];
            a[0] = 0b10;
            H256::from(a)
        });
        assert_eq!(vd3, {
            let mut a = [0; 32];
            a[0] = 0b01;
            H256::from(a)
        });

        let (ve31, vd31) = enabled_and_determined(&v3, 5);

        assert_eq!(ve31, {
            let mut a = [0; 32];
            a[0] = 0b11;
            H256::from(a)
        });
        assert_eq!(vd31, H256::zero());

        let v4 = vec![
            HardforkInfoInner {
                block_number: 0,
                flags:        {
                    let mut a = [0; 32];
                    a[0] = 0b10;
                    H256::from(a)
                },
            },
            HardforkInfoInner {
                block_number: 5,
                flags:        {
                    let mut a = [0; 32];
                    a[0] = 0b11;
                    H256::from(a)
                },
            },
            HardforkInfoInner {
                block_number: 10,
                flags:        {
                    let mut a = [0; 32];
                    a[0] = 0b111;
                    H256::from(a)
                },
            },
        ];

        let (ve4, vd4) = enabled_and_determined(&v4, 0);

        assert_eq!(ve4, {
            let mut a = [0; 32];
            a[0] = 0b10;
            H256::from(a)
        });
        assert_eq!(vd4, {
            let mut a = [0; 32];
            a[0] = 0b101;
            H256::from(a)
        });

        let (ve41, vd41) = enabled_and_determined(&v4, 2);

        assert_eq!(ve41, {
            let mut a = [0; 32];
            a[0] = 0b10;
            H256::from(a)
        });
        assert_eq!(vd41, {
            let mut a = [0; 32];
            a[0] = 0b101;
            H256::from(a)
        });

        let (ve42, vd42) = enabled_and_determined(&v4, 5);

        assert_eq!(ve42, {
            let mut a = [0; 32];
            a[0] = 0b11;
            H256::from(a)
        });
        assert_eq!(vd42, {
            let mut a = [0; 32];
            a[0] = 0b100;
            H256::from(a)
        });

        let (ve43, vd43) = enabled_and_determined(&v4, 8);

        assert_eq!(ve43, {
            let mut a = [0; 32];
            a[0] = 0b11;
            H256::from(a)
        });
        assert_eq!(vd43, {
            let mut a = [0; 32];
            a[0] = 0b100;
            H256::from(a)
        });

        let (ve44, vd44) = enabled_and_determined(&v4, 10);

        assert_eq!(ve44, {
            let mut a = [0; 32];
            a[0] = 0b111;
            H256::from(a)
        });
        assert_eq!(vd44, H256::zero());

        let (ve45, vd45) = enabled_and_determined(&v4, 12);

        assert_eq!(ve45, {
            let mut a = [0; 32];
            a[0] = 0b111;
            H256::from(a)
        });
        assert_eq!(vd45, H256::zero());
    }
}
