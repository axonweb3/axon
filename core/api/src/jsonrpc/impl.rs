use crate::adapter::DefaultAPIAdapter;
use crate::jsonrpc::{AxonJsonRpcServer, RpcResult};

use jsonrpsee::core::Error;

use protocol::traits::{APIAdapter, Context, MemPool, Storage};
use protocol::types::{
    BlockNumber, Bytes, SignedTransaction, UnverifiedTransaction, H160, H256, U256,
};
use protocol::{async_trait, codec::ProtocolCodec};

use crate::jsonrpc::types::{BlockId, CallRequest, RichTransactionOrHash, Web3Block};

pub struct JsonRpcImpl<M, S, DB> {
    adapter: DefaultAPIAdapter<M, S, DB>,
}

impl<M, S, DB> JsonRpcImpl<M, S, DB>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    pub fn new(adapter: DefaultAPIAdapter<M, S, DB>) -> Self {
        Self { adapter }
    }
}

#[async_trait]
impl<M, S, DB> AxonJsonRpcServer for JsonRpcImpl<M, S, DB>
where
    M: MemPool + 'static,
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    /// Sends signed transaction, returning its hash.
    async fn send_raw_transaction(&self, tx: Bytes) -> RpcResult<H256> {
        let utx = UnverifiedTransaction::decode(&tx[1..])
            .map_err(|e| Error::Custom(e.to_string()))?
            .hash();
        let stx = SignedTransaction::try_from(utx).map_err(|e| Error::Custom(e.to_string()))?;

        let hash = stx.transaction.hash;
        self.adapter
            .insert_signed_txs(Context::new(), stx)
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

    async fn get_block_by_number(
        &self,
        number: BlockId,
        show_rich_tx: bool,
    ) -> RpcResult<Option<Web3Block>> {
        let num = match number {
            BlockId::Num(n) => Some(n),
            _ => None,
        };

        let block = self
            .adapter
            .get_block_by_number(Context::new(), num)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        match block {
            Some(b) => {
                let capacity = b.tx_hashes.len();
                let mut ret = Web3Block::from(b);
                if show_rich_tx {
                    let mut txs = Vec::with_capacity(capacity);
                    for tx in ret.transactions.iter() {
                        let tx = self
                            .adapter
                            .get_transaction_by_hash(Context::new(), tx.get_hash())
                            .await
                            .map_err(|e| Error::Custom(e.to_string()))?
                            .unwrap();
                        txs.push(RichTransactionOrHash::Rich(tx));
                    }

                    ret.transactions = txs;
                }

                Ok(Some(ret))
            }
            None => Ok(None),
        }
    }

    async fn get_transaction_count(&self, address: H160, number: BlockId) -> RpcResult<U256> {
        let num = match number {
            BlockId::Num(n) => Some(n),
            _ => None,
        };

        let account = self
            .adapter
            .get_account(Context::new(), address, num)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(account.nonce)
    }

    async fn block_number(&self) -> RpcResult<BlockNumber> {
        self.adapter
            .get_latest_block(Context::new())
            .await
            .map(|b| b.header.number)
            .map_err(|e| Error::Custom(e.to_string()))
    }

    async fn get_balance(&self, address: H160, number: Option<BlockId>) -> RpcResult<U256> {
        let num = match number {
            Some(BlockId::Num(n)) => Some(n),
            _ => None,
        };

        let account = self
            .adapter
            .get_account(Context::new(), address, num)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(account.balance)
    }

    async fn chain_id(&self) -> RpcResult<U256> {
        self.adapter
            .get_latest_block(Context::new())
            .await
            .map(|b| b.header.chain_id.into())
            .map_err(|e| Error::Custom(e.to_string()))
    }

    async fn estimate_gas(&self, _req: CallRequest, number: Option<BlockId>) -> RpcResult<U256> {
        let _num = match number {
            Some(BlockId::Num(n)) => Some(n),
            _ => None,
        };

        Ok(Default::default())
    }
}
