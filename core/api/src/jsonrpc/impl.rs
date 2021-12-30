use jsonrpsee::types::Error;

use protocol::traits::{APIAdapter, Context, MemPool, Storage};
use protocol::types::{
    Bytes, Header, SignedTransaction, TxResp, UnverifiedTransaction, H160, H256, H64, U256,
};
use protocol::{async_trait, codec::ProtocolCodec, ProtocolResult};

use crate::jsonrpc::web3_types::{
    BlockId, RichTransactionOrHash, Web3Block, Web3CallRequest, Web3Receipt,
};
use crate::jsonrpc::{AxonJsonRpcServer, RpcResult};
use crate::{adapter::DefaultAPIAdapter, APIError};

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

    async fn call_evm(&self, req: Web3CallRequest, number: Option<u64>) -> ProtocolResult<TxResp> {
        let latest_header = self
            .adapter
            .get_block_header_by_number(Context::new(), number)
            .await?
            .ok_or_else(|| APIError::Storage(format!("Cannot get {:?} header", number)))?;

        let mock_header = mock_header_by_call_req(latest_header, &req);

        self.adapter
            .evm_call(Context::new(), req.from, req.data.to_vec(), mock_header)
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
        let block = self
            .adapter
            .get_block_by_number(Context::new(), number.into())
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
        let account = self
            .adapter
            .get_account(Context::new(), address, number.into())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(account.nonce)
    }

    async fn block_number(&self) -> RpcResult<U256> {
        self.adapter
            .get_block_header_by_number(Context::new(), None)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .map(|h| U256::from(h.number))
            .ok_or_else(|| Error::Custom("Cannot get latest block header".to_string()))
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
            .get_block_header_by_number(Context::new(), None)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .map(|h| U256::from(h.chain_id))
            .ok_or_else(|| Error::Custom("Cannot get latest block header".to_string()))
    }

    async fn net_version(&self) -> RpcResult<U256> {
        self.chain_id().await
    }

    async fn call(&self, req: Web3CallRequest) -> RpcResult<Vec<u8>> {
        let resp = self
            .call_evm(req, None)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(resp.ret)
    }

    async fn estimate_gas(&self, req: Web3CallRequest, number: Option<BlockId>) -> RpcResult<U256> {
        let num = match number {
            Some(BlockId::Num(n)) => Some(n),
            _ => None,
        };

        let resp = self
            .call_evm(req, num)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(resp.gas_used.into())
    }

    async fn get_code(&self, address: H160, number: BlockId) -> RpcResult<Vec<u8>> {
        let account = self
            .adapter
            .get_account(Context::new(), address, number.into())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;
        let code = self
            .adapter
            .get_code_by_hash(Context::new(), &account.code_hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .ok_or_else(|| {
                Error::Custom(format!("Cannot get code by hash {:?}", account.code_hash))
            })?;

        Ok(code.to_vec())
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

fn mock_header_by_call_req(latest_header: Header, call_req: &Web3CallRequest) -> Header {
    Header {
        prev_hash:         latest_header.prev_hash,
        proposer:          latest_header.proposer,
        state_root:        latest_header.state_root,
        transactions_root: Default::default(),
        signed_txs_hash:   Default::default(),
        receipts_root:     Default::default(),
        log_bloom:         Default::default(),
        difficulty:        latest_header.difficulty,
        timestamp:         latest_header.timestamp,
        number:            latest_header.number,
        gas_used:          latest_header.gas_used,
        gas_limit:         if let Some(gas_limit) = call_req.gas {
            gas_limit
        } else {
            latest_header.gas_limit
        },
        extra_data:        Default::default(),
        mixed_hash:        None,
        nonce:             if let Some(nonce) = call_req.nonce {
            H64::from_low_u64_le(nonce.as_u64())
        } else {
            latest_header.nonce
        },
        base_fee_per_gas:  if let Some(base_fee) = call_req.max_fee_per_gas {
            base_fee
        } else {
            latest_header.base_fee_per_gas
        },
        proof:             latest_header.proof,
        chain_id:          latest_header.chain_id,
    }
}
