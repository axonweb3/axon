use std::sync::Arc;

use jsonrpsee::core::Error;

use core_consensus::SYNC_STATUS;
use protocol::traits::{APIAdapter, Context};
use protocol::types::{
    Block, BlockNumber, Bytes, Hash, Hasher, Header, Hex, Receipt, SignedTransaction, TxResp,
    UnverifiedTransaction, H160, H256, H64, U256,
};
use protocol::{async_trait, codec::ProtocolCodec, ProtocolResult};
use std::time::Instant;

use crate::jsonrpc::web3_types::{
    BlockId, RichTransactionOrHash, Web3Block, Web3CallRequest, Web3FeeHistory, Web3Filter,
    Web3Log, Web3Receipt, Web3SyncStatus, Web3Transaction,
};
use crate::jsonrpc::{AxonJsonRpcServer, RpcResult};
use crate::APIError;
use common_apm::metrics::api::{
    API_REQUEST_RESULT_COUNTER_VEC_STATIC, API_REQUEST_TIME_HISTOGRAM_STATIC,
};

pub struct JsonRpcImpl<Adapter> {
    adapter: Arc<Adapter>,
    version: String,
}

impl<Adapter: APIAdapter> JsonRpcImpl<Adapter> {
    pub fn new(adapter: Arc<Adapter>, version: &str) -> Self {
        Self {
            adapter,
            version: version.to_string(),
        }
    }

    async fn call_evm(
        &self,
        req: Web3CallRequest,
        data: Bytes,
        number: Option<u64>,
    ) -> ProtocolResult<TxResp> {
        let header = self
            .adapter
            .get_block_header_by_number(Context::new(), number)
            .await?
            .ok_or_else(|| APIError::Storage(format!("Cannot get {:?} header", number)))?;

        let mock_header = mock_header_by_call_req(header, &req);

        self.adapter
            .evm_call(
                Context::new(),
                req.to,
                data.to_vec(),
                mock_header.state_root,
                mock_header.into(),
            )
            .await
    }
}

#[async_trait]
impl<Adapter: APIAdapter + 'static> AxonJsonRpcServer for JsonRpcImpl<Adapter> {
    #[metrics_rpc("eth_sendRawTransaction")]
    async fn send_raw_transaction(&self, tx: Hex) -> RpcResult<H256> {
        let utx = UnverifiedTransaction::decode(&tx.as_bytes()[1..])
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

    #[metrics_rpc("eth_getTransactionByHash")]
    async fn get_transaction_by_hash(&self, hash: H256) -> RpcResult<Option<Web3Transaction>> {
        let res = self
            .adapter
            .get_transaction_by_hash(Context::new(), hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        if let Some(stx) = res {
            if let Some(receipt) = self
                .adapter
                .get_receipt_by_tx_hash(Context::new(), hash)
                .await
                .map_err(|e| Error::Custom(e.to_string()))?
            {
                Ok(Some(Web3Transaction::create(receipt, stx)))
            } else {
                Err(Error::Custom(format!(
                    "can not get receipt by hash {:?}",
                    hash
                )))
            }
        } else {
            Ok(None)
        }
    }

    #[metrics_rpc("eth_getBlockByNumber")]
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

    #[metrics_rpc("eth_getTransactionCount")]
    async fn get_transaction_count(&self, address: H160, number: BlockId) -> RpcResult<U256> {
        let account = self
            .adapter
            .get_account(Context::new(), address, number.into())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(account.nonce)
    }

    #[metrics_rpc("eth_blockNumber")]
    async fn block_number(&self) -> RpcResult<U256> {
        self.adapter
            .get_block_header_by_number(Context::new(), None)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .map(|h| U256::from(h.number))
            .ok_or_else(|| Error::Custom("Cannot get latest block header".to_string()))
    }

    #[metrics_rpc("eth_getBalance")]
    async fn get_balance(&self, address: H160, number: BlockId) -> RpcResult<U256> {
        let account = self
            .adapter
            .get_account(Context::new(), address, number.into())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(account.balance)
    }

    #[metrics_rpc("eth_chainId")]
    async fn chain_id(&self) -> RpcResult<U256> {
        self.adapter
            .get_block_header_by_number(Context::new(), None)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .map(|h| U256::from(h.chain_id))
            .ok_or_else(|| Error::Custom("Cannot get latest block header".to_string()))
    }

    #[metrics_rpc("net_version")]
    async fn net_version(&self) -> RpcResult<U256> {
        self.chain_id().await
    }

    #[metrics_rpc("eth_call")]
    async fn call(&self, req: Web3CallRequest, number: BlockId) -> RpcResult<Hex> {
        let data_bytes = req.data.as_bytes();
        let resp = self
            .call_evm(req, data_bytes, number.into())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;
        let call_hex_result = Hex::encode(resp.ret);
        Ok(call_hex_result)
    }

    #[metrics_rpc("eth_estimateGas")]
    async fn estimate_gas(&self, req: Web3CallRequest, number: Option<BlockId>) -> RpcResult<U256> {
        let num = match number {
            Some(BlockId::Num(n)) => Some(n),
            _ => None,
        };
        let data_bytes = req.data.as_bytes();
        let resp = self
            .call_evm(req, data_bytes, num)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(resp.gas_used.into())
    }

    #[metrics_rpc("eth_getCode")]
    async fn get_code(&self, address: H160, number: BlockId) -> RpcResult<Hex> {
        let account = self
            .adapter
            .get_account(Context::new(), address, number.into())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        let code_result = self
            .adapter
            .get_code_by_hash(Context::new(), &account.code_hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;
        if let Some(code_bytes) = code_result {
            Ok(Hex::encode(code_bytes))
        } else {
            Ok(Hex::empty())
        }
    }

    #[metrics_rpc("eth_getBlockTransactionCountByNumber")]
    async fn get_transaction_count_by_number(&self, number: BlockId) -> RpcResult<U256> {
        let block = self
            .adapter
            .get_block_by_number(Context::new(), number.into())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;
        let count = match block {
            Some(bc) => bc.tx_hashes.len(),
            _ => 0,
        };
        Ok(U256::from(count))
    }

    #[metrics_rpc("eth_getTransactionReceipt")]
    async fn get_transaction_receipt(&self, hash: H256) -> RpcResult<Option<Web3Receipt>> {
        let res = self
            .adapter
            .get_transaction_by_hash(Context::new(), hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        if let Some(stx) = res {
            if let Some(receipt) = self
                .adapter
                .get_receipt_by_tx_hash(Context::new(), hash)
                .await
                .map_err(|e| Error::Custom(e.to_string()))?
            {
                Ok(Some(Web3Receipt::new(receipt, stx)))
            } else {
                Err(Error::Custom(format!(
                    "can not get receipt by hash {:?}",
                    hash
                )))
            }
        } else {
            Ok(None)
        }
    }

    #[metrics_rpc("eth_gasPrice")]
    async fn gas_price(&self) -> RpcResult<U256> {
        Ok(U256::from(8u64))
    }

    #[metrics_rpc("net_listening")]
    async fn listening(&self) -> RpcResult<bool> {
        Ok(true)
    }

    // #[metrics_rpc("net_peerCount")]
    async fn peer_count(&self) -> RpcResult<U256> {
        self.adapter
            .peer_count(Context::new())
            .await
            .map_err(|e| Error::Custom(e.to_string()))
    }

    // #[metrics_rpc("eth_syncing")]
    async fn syncing(&self) -> RpcResult<Web3SyncStatus> {
        Ok(SYNC_STATUS.read().clone().into())
    }

    #[metrics_rpc("eth_getLogs")]
    async fn get_logs(&self, filter: Web3Filter) -> RpcResult<Vec<Web3Log>> {
        if filter.topics.is_none() {
            return Ok(Vec::new());
        }

        let topics = filter.topics.unwrap();

        #[allow(clippy::large_enum_variant)]
        enum BlockPosition {
            Hash(H256),
            Num(BlockNumber),
            Block(Block),
        }

        async fn get_logs<T: APIAdapter>(
            adapter: &T,
            position: BlockPosition,
            topics: &[H256],
            logs: &mut Vec<Web3Log>,
        ) -> RpcResult<()> {
            let extend_logs = |logs: &mut Vec<Web3Log>, receipts: Vec<Option<Receipt>>| {
                let mut index = 0;
                for receipt in receipts.into_iter().flatten() {
                    let log_len = receipt.logs.len();
                    from_receipt_to_web3_log(index, topics, receipt, logs);
                    index += log_len;
                }
            };

            match position {
                BlockPosition::Hash(hash) => {
                    match adapter
                        .get_block_by_hash(Context::new(), hash)
                        .await
                        .map_err(|e| Error::Custom(e.to_string()))?
                    {
                        Some(block) => {
                            let receipts = adapter
                                .get_receipts_by_hashes(
                                    Context::new(),
                                    block.header.number,
                                    &block.tx_hashes,
                                )
                                .await
                                .map_err(|e| Error::Custom(e.to_string()))?;
                            extend_logs(logs, receipts);
                            Ok(())
                        }
                        None => Err(Error::Custom(format!(
                            "Invalid block hash
                    {}",
                            hash
                        ))),
                    }
                }
                BlockPosition::Num(n) => {
                    let block = adapter
                        .get_block_by_number(Context::new(), Some(n))
                        .await
                        .map_err(|e| Error::Custom(e.to_string()))?
                        .unwrap();
                    let receipts = adapter
                        .get_receipts_by_hashes(
                            Context::new(),
                            block.header.number,
                            &block.tx_hashes,
                        )
                        .await
                        .map_err(|e| Error::Custom(e.to_string()))?;

                    extend_logs(logs, receipts);
                    Ok(())
                }
                BlockPosition::Block(block) => {
                    let receipts = adapter
                        .get_receipts_by_hashes(
                            Context::new(),
                            block.header.number,
                            &block.tx_hashes,
                        )
                        .await
                        .map_err(|e| Error::Custom(e.to_string()))?;

                    extend_logs(logs, receipts);
                    Ok(())
                }
            }
        }

        let mut all_logs = Vec::new();
        match filter.block_hash {
            Some(hash) => {
                get_logs(
                    &*self.adapter,
                    BlockPosition::Hash(hash),
                    &topics,
                    &mut all_logs,
                )
                .await?;
            }
            None => {
                let latest_block = self
                    .adapter
                    .get_block_by_number(Context::new(), None)
                    .await
                    .map_err(|e| Error::Custom(e.to_string()))?
                    .unwrap();
                let latest_number = latest_block.header.number;
                let (start, end) = {
                    let convert = |id: BlockId| -> BlockNumber {
                        match id {
                            BlockId::Num(n) => n,
                            BlockId::Latest => latest_number,
                        }
                    };

                    (
                        filter.from_block.map(convert).unwrap_or(latest_number),
                        filter.to_block.map(convert).unwrap_or(latest_number),
                    )
                };

                if start > latest_number {
                    return Err(Error::Custom(format!("Invalid from_block {}", start)));
                }

                let mut visiter_last_block = false;
                for n in start..=end {
                    if n == latest_number {
                        visiter_last_block = true;
                    } else {
                        get_logs(
                            &*self.adapter,
                            BlockPosition::Num(n),
                            &topics,
                            &mut all_logs,
                        )
                        .await?;
                    }
                }

                if visiter_last_block {
                    get_logs(
                        &*self.adapter,
                        BlockPosition::Block(latest_block),
                        &topics,
                        &mut all_logs,
                    )
                    .await?;
                }
            }
        }
        Ok(all_logs)
    }

    async fn fee_history(
        &self,
        _block_count: u64,
        _newest_block: BlockId,
        _reward_percentiles: Option<Vec<u64>>,
    ) -> RpcResult<Web3FeeHistory> {
        Ok(Web3FeeHistory {
            oldest_block:     U256::from(0),
            reward:           None,
            base_fee_per_gas: Vec::new(),
            gas_used_ratio:   Vec::new(),
        })
    }

    async fn client_version(&self) -> RpcResult<String> {
        Ok(self.version.clone())
    }

    async fn accounts(&self) -> RpcResult<Vec<Hex>> {
        Ok(vec![])
    }

    async fn sha3(&self, data: Hex) -> RpcResult<Hash> {
        let decode_data =
            Hex::decode(data.as_string()).map_err(|e| Error::Custom(e.to_string()))?;
        let ret = Hasher::digest(decode_data.as_ref());
        Ok(ret)
    }
}

fn mock_header_by_call_req(latest_header: Header, call_req: &Web3CallRequest) -> Header {
    Header {
        prev_hash:                  latest_header.prev_hash,
        proposer:                   latest_header.proposer,
        state_root:                 latest_header.state_root,
        transactions_root:          Default::default(),
        signed_txs_hash:            Default::default(),
        receipts_root:              Default::default(),
        log_bloom:                  Default::default(),
        difficulty:                 latest_header.difficulty,
        timestamp:                  latest_header.timestamp,
        number:                     latest_header.number,
        gas_used:                   latest_header.gas_used,
        gas_limit:                  if let Some(gas_limit) = call_req.gas {
            gas_limit
        } else {
            latest_header.gas_limit
        },
        extra_data:                 Default::default(),
        mixed_hash:                 None,
        nonce:                      if let Some(nonce) = call_req.nonce {
            H64::from_low_u64_le(nonce.as_u64())
        } else {
            latest_header.nonce
        },
        base_fee_per_gas:           if let Some(base_fee) = call_req.max_fee_per_gas {
            base_fee
        } else {
            latest_header.base_fee_per_gas
        },
        proof:                      latest_header.proof,
        last_checkpoint_block_hash: latest_header.last_checkpoint_block_hash,
        chain_id:                   latest_header.chain_id,
    }
}

fn from_receipt_to_web3_log(
    index: usize,
    topics: &[H256],
    receipt: Receipt,
    logs: &mut Vec<Web3Log>,
) {
    for log in receipt.logs {
        for (idx, topic) in log.topics.iter().enumerate() {
            if topics.contains(topic) {
                let web3_log = Web3Log {
                    address:           receipt.sender,
                    topics:            log.topics.clone(),
                    data:              Hex::encode(&log.data),
                    block_hash:        Some(receipt.block_hash),
                    block_number:      Some(receipt.block_number.into()),
                    transaction_hash:  Some(receipt.tx_hash),
                    transaction_index: Some(receipt.tx_index.into()),
                    log_index:         Some((index + idx).into()),
                    removed:           false,
                };
                logs.push(web3_log);
            }
        }
    }
}
