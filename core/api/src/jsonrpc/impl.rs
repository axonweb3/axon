use jsonrpsee::types::Error;

use protocol::traits::{APIAdapter, Context, MemPool, Storage};
use protocol::types::{
    TransactionAction,
    ExitReason, ExitSucceed, Hasher, Hex, SignedTransaction, UnverifiedTransaction, H160,
    H256, U256,
};
use protocol::{async_trait, codec::ProtocolCodec};

use crate::adapter::DefaultAPIAdapter;
use crate::jsonrpc::types::{
    BlockId, RichTransactionOrHash, Web3Block, Web3CallRequest, Web3EstimateRequst,
    Web3SendTrancationRequest, Web3TransactionReceipt,
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

    // async fn sign(&self, address: H160, data: Bytes) ->
    // RpcResult<Option<Vec<u8>>> {     todo!()
    // }

    /// Sends signed transaction, returning its hash.
    async fn send_raw_transaction(&self, tx: String) -> RpcResult<H256> {
        println!("transaction：{:?}", &tx);
        let txx = Hex::from_string(tx)
            .map_err(|e| Error::Custom(e.to_string()))?
            .decode();
        // let txx=tx.as_bytes();
        let utx = UnverifiedTransaction::decode(&txx[1..])
            .map_err(|e| Error::Custom(e.to_string()))?
            .hash();
        let mut stx = SignedTransaction::try_from(utx).map_err(|e| Error::Custom(e.to_string()))?;
        // stx.transaction.unsigned.action= TransactionAction::Create;
        let hash = stx.transaction.hash;
        self.adapter
            .insert_signed_txs(Context::new(), stx)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(hash)
    }

    async fn send_transaction(&self, tx: Web3SendTrancationRequest) -> RpcResult<Option<H256>> {
        // let tx = SignedTransaction::decode(tx.data).map_err(|e|
        // Error::Custom(e.to_string()))?;
        let mut txx = tx.create_signedtransaction_by_web3sendtrancationrequest();
        txx.transaction = txx.transaction.hash();
        self.adapter
            .insert_signed_txs(Context::new(), txx.clone())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;
        Ok(Some(txx.transaction.hash))
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
        // Ok(1.into())
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
        Ok(U256::from("539"))
        // self.adapter
        //     .get_latest_block(Context::new())
        //     .await
        //     .map(|b| b.header.chain_id.into())
        //     .map_err(|e| Error::Custom(e.to_string()))
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
        let block;
        let uncodestr = "0x0";
        match number {
            Some(num) => {
                block = self
                    .adapter
                    .get_block_by_number(Context::new(), Some(num))
                    .await
                    .map_err(|e| Error::Custom(e.to_string()))?;
            }
            _ => {
                block = Some(
                    self.adapter
                        .get_latest_block(Context::new())
                        .await
                        .map_err(|e| Error::Custom(e.to_string()))?,
                )
            }
        };

        let codebytes = match block {
            Some(b) => {
                let ret = Web3Block::from(b);
                let mytruascations: Vec<RichTransactionOrHash> = ret
                    .transactions
                    .into_iter()
                    .filter(|item| match item {
                        RichTransactionOrHash::Hash(_) => false,
                        RichTransactionOrHash::Rich(stx) => {
                            if stx.sender == address {
                                true
                            } else {
                                false
                            }
                        }
                    })
                    .map(|v| v)
                    .collect();
                if mytruascations.len() <= 0 {
                    uncodestr.as_bytes().to_vec()
                } else {
                    let mut data: Vec<Vec<u8>> = vec![];
                    for tx in mytruascations {
                        if let RichTransactionOrHash::Rich(st) = tx {
                            let datahash = Hasher::digest(st.transaction.unsigned.data);
                            let code = self
                                .adapter
                                .get_code_by_hash(Context::new(), &datahash)
                                .await
                                .map_err(|e| Error::Custom(e.to_string()))?;
                            if let Some(c) = code {
                                data.push(c.to_vec());
                            } else {
                                // vec![]//  data.push();
                            }
                        }
                    }
                    if data.len() <= 0 {
                        data.push(uncodestr.as_bytes().to_vec());
                    }
                    data.get(0).unwrap().clone() // "0x0".as_bytes().to_vec()
                }
            }
            None => uncodestr.as_bytes().to_vec(),
        };
        Ok(codebytes)
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

    async fn get_transaction_receipt(
        &self,
        _hash: H256,
    ) -> RpcResult<Option<Web3TransactionReceipt>> {
        let tx = self
            .adapter
            .get_transaction_by_hash(Context::new(), _hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        let rp = self
            .adapter
            .get_receipt_by_tx_hash(Context::new(), _hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        match tx {
            Some(y) => rp.map_or_else(
                move || Ok(None),
                |v| Ok(Some(Web3TransactionReceipt::create_new(v, y))),
            ),
            None => Ok(None),
        }
    }

    async fn get_gas_price(&self) -> RpcResult<Option<U256>> {
        Ok(Some(U256::from("8")))
    }
}
