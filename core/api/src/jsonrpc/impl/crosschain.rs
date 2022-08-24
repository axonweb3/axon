use std::sync::Arc;

use protocol::traits::{APIAdapter, Context};
use protocol::{async_trait, types::H256};

use crate::jsonrpc::web3_types::{Web3Receipt, Web3Transaction};
use crate::jsonrpc::{
    crosschain_types::CrossChainTransaction, AxonCrossChainRpcServer, Error, RpcResult,
};

pub struct CrossChainRpcImpl<Adapter> {
    adapter: Arc<Adapter>,
}

impl<Adapter: APIAdapter> CrossChainRpcImpl<Adapter> {
    pub fn new(adapter: Arc<Adapter>) -> Self {
        CrossChainRpcImpl { adapter }
    }
}

#[async_trait]
impl<Adapter: APIAdapter + 'static> AxonCrossChainRpcServer for CrossChainRpcImpl<Adapter> {
    async fn get_crosschain_result(
        &self,
        tx_hash: H256,
    ) -> RpcResult<Option<CrossChainTransaction>> {
        let ctx = Context::new();
        if let Some(hash_with_dir) = self
            .adapter
            .get_crosschain_record_by_hash(ctx.clone(), &tx_hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
        {
            if !hash_with_dir.direction.is_from_ckb() {
                return Ok(Some(CrossChainTransaction {
                    request_tx_hash: tx_hash,
                    relay_tx_hash:   hash_with_dir.tx_hash,
                    direction:       hash_with_dir.direction,
                    axon_tx:         None,
                    receipt:         None,
                }));
            }

            let hash = hash_with_dir.tx_hash;
            let stx = self
                .adapter
                .get_transaction_by_hash(ctx.clone(), hash)
                .await
                .map_err(|e| Error::Custom(e.to_string()))?
                .ok_or_else(|| {
                    Error::Custom(format!("Can not get transaction by hash {:?}", hash))
                })?;
            let receipt = self
                .adapter
                .get_receipt_by_tx_hash(ctx, hash)
                .await
                .map_err(|e| Error::Custom(e.to_string()))?
                .ok_or_else(|| Error::Custom(format!("Can not get receipt by hash {:?}", hash)))?;

            return Ok(Some(CrossChainTransaction {
                request_tx_hash: tx_hash,
                relay_tx_hash:   hash,
                direction:       hash_with_dir.direction,
                axon_tx:         Some(Web3Transaction::from((stx.clone(), receipt.clone()))),
                receipt:         Some(Web3Receipt::new(receipt, stx)),
            }));
        }

        Ok(None)
    }
}
