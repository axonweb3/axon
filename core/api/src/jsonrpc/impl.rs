use jsonrpsee::types::Error;

use protocol::traits::{APIAdapter, Context, MemPool, Storage};
use protocol::types::{
    Bytes, ExitReason, ExitSucceed, Hasher, SignedTransaction, UnverifiedTransaction, H160, H256,
    U256,
};
use protocol::{async_trait, codec::ProtocolCodec};

use crate::adapter::DefaultAPIAdapter;
use crate::jsonrpc::types::{
    BlockId, RichTransactionOrHash, Web3Block, Web3CallRequest, Web3EstimateRequst, Web3Receipt,
};
use crate::jsonrpc::{AxonJsonRpcServer, RpcResult};

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
    async fn listening(&self) -> RpcResult<bool> {
        Ok(true)
    }

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

    async fn block_number(&self) -> RpcResult<U256> {
        self.adapter
            .get_latest_block(Context::new())
            .await
            .map(|b| U256::from(b.header.number))
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

    async fn net_version(&self) -> RpcResult<U256> {
        self.chain_id().await
    }

    async fn call(&self, w3crequest: Web3CallRequest) -> RpcResult<Option<Vec<u8>>> {
        let gentx = w3crequest.create_signedtransaction_by_web3allrequest();
        let rpx = self.adapter.evm_call(gentx).await;
        if rpx.exit_reason != ExitReason::Succeed(ExitSucceed::Returned) {
            Ok(None)
        } else {
            Ok(Some(rpx.ret))
        }
    }

    async fn get_code(&self, address: H160, number: Option<u64>) -> RpcResult<Vec<u8>> {
        let block = self
            .adapter
            .get_block_by_number(Context::new(), number)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .ok_or_else(|| Error::Custom("Cannot get block".to_string()))?;

        let receipts = self
            .adapter
            .get_receipts_by_hashes(Context::new(), block.header.number, &block.tx_hashes)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        if receipts.len() != block.tx_hashes.len() {
            return Err(Error::Custom("Missing transaction".to_string()));
        }

        for receipt in receipts.iter() {
            if receipt.sender == address && receipt.code_address.is_some() {
                let stx = self
                    .adapter
                    .get_transaction_by_hash(Context::new(), receipt.tx_hash)
                    .await
                    .map_err(|e| Error::Custom(e.to_string()))?
                    .ok_or_else(|| {
                        Error::Custom(format!(
                            "Cannot get transaction by hash {:?}",
                            receipt.tx_hash
                        ))
                    })?;
                return Ok(Hasher::digest(&stx.transaction.unsigned.data)
                    .as_bytes()
                    .to_vec());
            }
        }

        Ok(Vec::new())
    }

    async fn estimate_gas(&self, req: Web3EstimateRequst) -> RpcResult<Option<U256>> {
        let gentx = req.create_signedtransaction_by_web3estimaterequst();
        let rpx = self.adapter.evm_call(gentx).await;
        Ok(Some(U256::from(rpx.remain_gas)))
        // Ok(Some(rpx.re.into()))
        // if rpx.exit_reason != ExitReason::Succeed(ExitSucceed::Stopped) {
        //     Ok(None)
        // } else {
        //     Ok(Some(rpx.gas_used))
        // }
    }

    async fn get_transaction_receipt(&self, hash: H256) -> RpcResult<Option<Web3Receipt>> {
        let res = self
            .adapter
            .get_transaction_by_hash(Context::new(), hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        if res.is_none() {
            return Ok(None);
        }

        let stx = res.unwrap();
        if let Some(receipt) = self
            .adapter
            .get_receipt_by_tx_hash(Context::new(), hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
        {
            Ok(Some(Web3Receipt::new(receipt, stx)))
        } else {
            Err(Error::Custom(format!(
                "Cannot get receipt by hash {:?}",
                hash
            )))
        }
    }

    async fn get_gas_price(&self) -> RpcResult<Option<U256>> {
        Ok(Some(U256::from(8u64)))
    }
}
