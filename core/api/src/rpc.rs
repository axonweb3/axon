use crate::{adapter::Adapter, AxonRpcServer, RpcResult};

use jsonrpsee::types::Error;
use protocol::traits::APIAdapter;
use protocol::{
    async_trait,
    codec::ProtocolCodec,
    traits::{Context, MemPool, Storage},
    types::{BlockNumber, Bytes, RichBlock, SignedTransaction, H256},
};

pub struct RpcImpl<M, S> {
    adapter: Adapter<M, S>,
}

impl<M, S> RpcImpl<M, S>
where
    M: MemPool + 'static,
    S: Storage + 'static,
{
    pub fn new(adapter: Adapter<M, S>) -> Self {
        Self { adapter }
    }
}

#[async_trait]
impl<M, S> AxonRpcServer for RpcImpl<M, S>
where
    M: MemPool + 'static,
    S: Storage + 'static,
{
    /// Sends signed transaction, returning its hash.
    async fn send_raw_transaction(&self, tx: Bytes) -> RpcResult<H256> {
        let tx = SignedTransaction::decode(tx).map_err(|e| Error::Custom(e.to_string()))?;
        let hash = tx.transaction.hash;
        self.adapter
            .insert_signed_txs(Context::new(), tx)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(hash)
    }

    /// Get transaction by its hash.
    async fn get_transaction_by_hash(&self, hash: H256) -> RpcResult<SignedTransaction> {
        let tx = self
            .adapter
            .get_transaction_by_hash(Context::new(), hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        tx.ok_or_else(|| Error::Custom("Can't find this transaction".to_string()))
    }

    async fn block_by_number(
        &self,
        number: BlockNumber,
        _ignore: bool,
    ) -> RpcResult<Option<RichBlock>> {
        let block = self
            .adapter
            .get_block_by_height(Context::new(), Some(number))
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        match block {
            Some(b) => {
                let mut txs = Vec::with_capacity(b.tx_hashes.len());
                for hash in b.tx_hashes.iter() {
                    let tx = self
                        .adapter
                        .get_transaction_by_hash(Context::new(), *hash)
                        .await
                        .map_err(|e| Error::Custom(e.to_string()))?
                        .unwrap();
                    txs.push(tx);
                }

                Ok(Some(RichBlock { block: b, txs }))
            }
            None => Ok(None),
        }
    }
}
