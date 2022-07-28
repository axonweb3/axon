use std::sync::Arc;

use jsonrpsee::core::Error;

use common_apm::metrics_rpc;
use protocol::traits::{APIAdapter, Context};
use protocol::types::{
    Block, BlockNumber, Bytes, Hash, Header, Hex, Receipt, SignedTransaction, TxResp,
    UnverifiedTransaction, H160, H256, H64, MAX_BLOCK_GAS_LIMIT, MAX_CONTRACT_CODE_SIZE,
    MIN_TRANSACTION_GAS_LIMIT, U256,
};
use protocol::{async_trait, codec::ProtocolCodec, ProtocolResult};

use crate::jsonrpc::web3_types::{
    BlockId, RichTransactionOrHash, Web3Block, Web3CallRequest, Web3FeeHistory, Web3Filter,
    Web3Log, Web3Receipt, Web3Transaction,
};
use crate::jsonrpc::{error::RpcError, AxonWeb3RpcServer, RpcResult};
use crate::APIError;

pub(crate) const MAX_LOG_NUM: usize = 10000;

pub struct Web3RpcImpl<Adapter> {
    adapter: Arc<Adapter>,
    gas_cap: U256,
}

impl<Adapter: APIAdapter> Web3RpcImpl<Adapter> {
    pub fn new(adapter: Arc<Adapter>, gas_cap: u64) -> Self {
        Self {
            adapter,
            gas_cap: gas_cap.into(),
        }
    }

    async fn call_evm(
        &self,
        req: Web3CallRequest,
        data: Bytes,
        number: Option<u64>,
    ) -> ProtocolResult<TxResp> {
        if req.from.is_none() && req.to.is_none() {
            return Err(APIError::RequestPayload("from and to are both None".to_string()).into());
        }

        let header = self
            .adapter
            .get_block_header_by_number(Context::new(), number)
            .await?
            .ok_or_else(|| APIError::Storage(format!("Cannot get {:?} header", number)))?;

        let mock_header = mock_header_by_call_req(header, &req);

        self.adapter
            .evm_call(
                Context::new(),
                req.from,
                req.to,
                req.gas_price,
                req.gas,
                req.value.unwrap_or_default(),
                data.to_vec(),
                mock_header.state_root,
                mock_header.into(),
            )
            .await
    }
}

#[async_trait]
impl<Adapter: APIAdapter + 'static> AxonWeb3RpcServer for Web3RpcImpl<Adapter> {
    #[metrics_rpc("eth_sendRawTransaction")]
    async fn send_raw_transaction(&self, tx: Hex) -> RpcResult<H256> {
        let utx = UnverifiedTransaction::decode(&tx.as_bytes())
            .map_err(|e| Error::Custom(e.to_string()))?;

        if utx.unsigned.data().len() > MAX_CONTRACT_CODE_SIZE {
            return Err(Error::Custom("The data exceeds the max size".to_string()));
        }

        let gas_price = utx.unsigned.gas_price();

        if gas_price == U256::zero() {
            return Err(Error::Custom(
                "The transaction gas price is zero".to_string(),
            ));
        }

        if gas_price >= U256::from(u64::MAX) {
            return Err(Error::Custom("The gas price is too large".to_string()));
        }

        let gas_limit = *utx.unsigned.gas_limit();

        if gas_limit < MIN_TRANSACTION_GAS_LIMIT.into() {
            return Err(Error::Custom(
                "The transaction gas limit less than 21000".to_string(),
            ));
        }

        if gas_limit > self.gas_cap {
            return Err(Error::Custom(
                "The transaction gas limit is too large".to_string(),
            ));
        }

        utx.check_hash().map_err(|e| Error::Custom(e.to_string()))?;

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
                Ok(Some((stx, receipt).into()))
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
                let block_number = b.header.number;
                let block_hash = b.hash();
                let mut ret = Web3Block::from(b);
                if show_rich_tx {
                    let mut txs = Vec::with_capacity(capacity);
                    for (idx, tx) in ret.transactions.iter().enumerate() {
                        let tx = self
                            .adapter
                            .get_transaction_by_hash(Context::new(), tx.get_hash())
                            .await
                            .map_err(|e| Error::Custom(e.to_string()))?
                            .unwrap();

                        txs.push(RichTransactionOrHash::Rich(
                            Web3Transaction::from(tx)
                                .add_block_number(block_number)
                                .add_block_hash(block_hash)
                                .add_tx_index(idx),
                        ));
                    }

                    ret.transactions = txs;
                }

                Ok(Some(ret))
            }
            None => Ok(None),
        }
    }

    #[metrics_rpc("eth_getBlockByHash")]
    async fn get_block_by_hash(
        &self,
        hash: H256,
        show_rich_tx: bool,
    ) -> RpcResult<Option<Web3Block>> {
        let block = self
            .adapter
            .get_block_by_hash(Context::new(), hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        match block {
            Some(b) => {
                let capacity = b.tx_hashes.len();
                let block_number = b.header.number;
                let block_hash = b.hash();
                let mut ret = Web3Block::from(b);
                if show_rich_tx {
                    let mut txs = Vec::with_capacity(capacity);
                    for (idx, tx) in ret.transactions.iter().enumerate() {
                        let tx = self
                            .adapter
                            .get_transaction_by_hash(Context::new(), tx.get_hash())
                            .await
                            .map_err(|e| Error::Custom(e.to_string()))?
                            .unwrap();

                        txs.push(RichTransactionOrHash::Rich(
                            Web3Transaction::from(tx)
                                .add_block_number(block_number)
                                .add_block_hash(block_hash)
                                .add_tx_index(idx),
                        ));
                    }

                    ret.transactions = txs;
                }

                Ok(Some(ret))
            }
            None => Ok(None),
        }
    }

    #[metrics_rpc("eth_getTransactionCount")]
    async fn get_transaction_count(
        &self,
        address: H160,
        number: Option<BlockId>,
    ) -> RpcResult<U256> {
        match number.unwrap_or_default() {
            BlockId::Pending => {
                let pending_tx_count = self
                    .adapter
                    .get_pending_tx_count(Context::new(), address)
                    .await
                    .map_err(|e| Error::Custom(e.to_string()))?;
                Ok(self
                    .adapter
                    .get_account(Context::new(), address, BlockId::Pending.into())
                    .await
                    .map(|account| account.nonce + pending_tx_count)
                    .unwrap_or_default())
            }
            b => Ok(self
                .adapter
                .get_account(Context::new(), address, b.into())
                .await
                .map(|account| account.nonce)
                .unwrap_or_default()),
        }
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
    async fn get_balance(&self, address: H160, number: Option<BlockId>) -> RpcResult<U256> {
        Ok(self
            .adapter
            .get_account(Context::new(), address, number.unwrap_or_default().into())
            .await
            .map_or(U256::zero(), |account| account.balance))
    }

    #[metrics_rpc("eth_call")]
    async fn call(&self, req: Web3CallRequest, number: Option<BlockId>) -> RpcResult<Hex> {
        if req.gas_price.unwrap_or_default() > U256::from(u64::MAX) {
            return Err(Error::Custom("The gas price is too large".to_string()));
        }

        if req.gas.unwrap_or_default() > U256::from(MAX_BLOCK_GAS_LIMIT) {
            return Err(Error::Custom("The gas limit is too large".to_string()));
        }

        let data_bytes = req
            .data
            .as_ref()
            .map(|hex| hex.as_bytes())
            .unwrap_or_default();
        let resp = self
            .call_evm(req, data_bytes, number.unwrap_or_default().into())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        if resp.exit_reason.is_succeed() {
            let call_hex_result = Hex::encode(resp.ret);
            return Ok(call_hex_result);
        }

        Err(RpcError::VM(resp).into())
    }

    #[metrics_rpc("eth_estimateGas")]
    async fn estimate_gas(&self, req: Web3CallRequest, number: Option<BlockId>) -> RpcResult<U256> {
        if let Some(gas_limit) = req.gas.as_ref() {
            if gas_limit == &U256::zero() {
                return Err(Error::Custom("Failed: Gas cannot be zero".to_string()));
            }
        }

        if let Some(price) = req.gas_price.as_ref() {
            if price >= &U256::from(u64::MAX) {
                return Err(Error::Custom("Failed: Gas price too high".to_string()));
            }
        }

        let num = match number {
            Some(BlockId::Num(n)) => Some(n),
            _ => None,
        };
        let data_bytes = req
            .data
            .as_ref()
            .map(|hex| hex.as_bytes())
            .unwrap_or_default();
        let resp = self
            .call_evm(req, data_bytes, num)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        if resp.exit_reason.is_succeed() {
            return Ok(resp.gas_used.into());
        }

        Err(RpcError::VM(resp).into())
    }

    #[metrics_rpc("eth_getCode")]
    async fn get_code(&self, address: H160, number: Option<BlockId>) -> RpcResult<Hex> {
        let account = self
            .adapter
            .get_account(Context::new(), address, number.unwrap_or_default().into())
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
    async fn get_block_transaction_count_by_number(&self, number: BlockId) -> RpcResult<U256> {
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

    #[metrics_rpc("net_peerCount")]
    async fn peer_count(&self) -> RpcResult<U256> {
        self.adapter
            .peer_count(Context::new())
            .await
            .map_err(|e| Error::Custom(e.to_string()))
    }

    #[metrics_rpc("eth_gasPrice")]
    async fn gas_price(&self) -> RpcResult<U256> {
        Ok(U256::from(8u64))
    }

    #[metrics_rpc("eth_getLogs")]
    async fn get_logs(&self, filter: Web3Filter) -> RpcResult<Vec<Web3Log>> {
        let topics: Vec<Option<Vec<Option<H256>>>> = filter
            .topics
            .map(|s| {
                s.into_iter()
                    .take(4)
                    .map(Into::<Option<Vec<Option<H256>>>>::into)
                    .collect()
            })
            .unwrap_or_default();

        #[allow(clippy::large_enum_variant)]
        enum BlockPosition {
            Hash(H256),
            Num(BlockNumber),
            Block(Block),
        }

        async fn get_logs<T: APIAdapter>(
            adapter: &T,
            position: BlockPosition,
            topics: &[Option<Vec<Option<H256>>>],
            logs: &mut Vec<Web3Log>,
            address: Option<&Vec<H160>>,
            early_return: &mut bool,
        ) -> RpcResult<()> {
            let extend_logs = |logs: &mut Vec<Web3Log>,
                               receipts: Vec<Option<Receipt>>,
                               early_return: &mut bool| {
                for (index, receipt) in receipts.into_iter().flatten().enumerate() {
                    match address {
                        Some(s)
                            if s.contains(
                                &receipt.code_address.map(Into::into).unwrap_or_default(),
                            ) =>
                        {
                            from_receipt_to_web3_log(index, topics, &receipt, logs)
                        }
                        None => from_receipt_to_web3_log(index, topics, &receipt, logs),
                        _ => (),
                    }
                    if logs.len() > MAX_LOG_NUM {
                        *early_return = true;
                        return;
                    }
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
                            extend_logs(logs, receipts, early_return);
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

                    extend_logs(logs, receipts, early_return);
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

                    extend_logs(logs, receipts, early_return);
                    Ok(())
                }
            }
        }

        let address_filter: Option<Vec<H160>> = filter.address.into();
        let mut all_logs = Vec::new();
        let mut early_return = false;
        match filter.block_hash {
            Some(hash) => {
                get_logs(
                    &*self.adapter,
                    BlockPosition::Hash(hash),
                    &topics,
                    &mut all_logs,
                    address_filter.as_ref(),
                    &mut early_return,
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
                            BlockId::Earliest => 0,
                            _ => latest_number,
                        }
                    };

                    (
                        filter.from_block.map(convert).unwrap_or(latest_number),
                        std::cmp::min(
                            filter.to_block.map(convert).unwrap_or(latest_number),
                            latest_number,
                        ),
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
                            address_filter.as_ref(),
                            &mut early_return,
                        )
                        .await?;

                        if early_return {
                            return Ok(all_logs);
                        }
                    }
                }

                if visiter_last_block {
                    get_logs(
                        &*self.adapter,
                        BlockPosition::Block(latest_block),
                        &topics,
                        &mut all_logs,
                        address_filter.as_ref(),
                        &mut early_return,
                    )
                    .await?;
                }
            }
        }
        Ok(all_logs)
    }

    #[metrics_rpc("eth_feeHistory")]
    async fn fee_history(
        &self,
        _block_count: U256,
        _newest_block: BlockId,
        _reward_percentiles: Option<Vec<f64>>,
    ) -> RpcResult<Web3FeeHistory> {
        Ok(Web3FeeHistory {
            oldest_block:     U256::from(0),
            reward:           None,
            base_fee_per_gas: Vec::new(),
            gas_used_ratio:   Vec::new(),
        })
    }

    #[metrics_rpc("eth_accounts")]
    async fn accounts(&self) -> RpcResult<Vec<Hex>> {
        Ok(vec![])
    }

    #[metrics_rpc("eth_getBlockTransactionCountByHash")]
    async fn get_block_transaction_count_by_hash(&self, hash: Hash) -> RpcResult<U256> {
        Ok(self
            .adapter
            .get_block_by_hash(Context::new(), hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .map(|b| U256::from(b.tx_hashes.len()))
            .unwrap_or_default())
    }

    #[metrics_rpc("eth_getTransactionByBlockHashAndIndex")]
    async fn get_transaction_by_block_hash_and_index(
        &self,
        hash: Hash,
        position: U256,
    ) -> RpcResult<Option<Web3Transaction>> {
        if position > U256::from(usize::MAX) {
            return Err(Error::Custom(format!("invalid position: {}", position)));
        }

        let mut raw = [0u8; 32];

        position.to_little_endian(&mut raw);

        let mut raw_index = [0u8; 8];
        raw_index.copy_from_slice(&raw[..8]);
        let index = usize::from_le_bytes(raw_index);
        let block = self
            .adapter
            .get_block_by_hash(Context::new(), hash)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        if let Some(block) = block {
            if let Some(tx_hash) = block.tx_hashes.get(index) {
                return self.get_transaction_by_hash(*tx_hash).await;
            }
        }
        Ok(None)
    }

    #[metrics_rpc("eth_getTransactionByBlockNumberAndIndex")]
    async fn get_transaction_by_block_number_and_index(
        &self,
        number: BlockId,
        position: U256,
    ) -> RpcResult<Option<Web3Transaction>> {
        if position > U256::from(usize::MAX) {
            return Err(Error::Custom(format!("invalid position: {}", position)));
        }

        let mut raw = [0u8; 32];

        position.to_little_endian(&mut raw);

        let mut raw_index = [0u8; 8];
        raw_index.copy_from_slice(&raw[..8]);
        let index = usize::from_le_bytes(raw_index);

        let block = self
            .adapter
            .get_block_by_number(Context::new(), number.into())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        if let Some(block) = block {
            if let Some(tx_hash) = block.tx_hashes.get(index) {
                return self.get_transaction_by_hash(*tx_hash).await;
            }
        }
        Ok(None)
    }

    #[metrics_rpc("eth_getStorageAt")]
    async fn get_storage_at(
        &self,
        address: H160,
        position: U256,
        number: Option<BlockId>,
    ) -> RpcResult<Hex> {
        let block = self
            .adapter
            .get_block_by_number(Context::new(), number.unwrap_or_default().into())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .ok_or_else(|| Error::Custom("Can't find this block".to_string()))?;
        let value = self
            .adapter
            .get_storage_at(Context::new(), address, position, block.header.state_root)
            .await
            .unwrap_or_else(|_| H256::default().as_bytes().to_vec().into());

        Ok(Hex::encode(&value))
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
        call_system_script_count:   0,
        chain_id:                   latest_header.chain_id,
    }
}

pub fn from_receipt_to_web3_log(
    index: usize,
    topics: &[Option<Vec<Option<Hash>>>],
    receipt: &Receipt,
    logs: &mut Vec<Web3Log>,
) {
    macro_rules! contains_topic {
        ($topics: expr, $log: expr) => {{
            $topics.is_empty()
                || contains_topic!($topics, 1, $log, 0)
                || contains_topic!($topics, 2, $log, 0, 1)
                || contains_topic!($topics, 3, $log, 0, 1, 2)
                || contains_topic!($topics, 4, $log, 0, 1, 2, 3)
        }};

        ($topics: expr, $min_len: expr, $log: expr$ (, $offset: expr)*) => {{
            $topics.len() == $min_len && $log.topics.len() >= $min_len
            $( && $topics[$offset]
                .as_ref()
                .map(|i| i.contains(&None) || i.contains(&Some($log.topics[$offset])))
                .unwrap_or(true)
            )*
        }};
    }

    for (log_idex, log) in receipt.logs.iter().enumerate() {
        if contains_topic!(topics, log) {
            let web3_log = Web3Log {
                address:           receipt.sender,
                topics:            log.topics.clone(),
                data:              Hex::encode(&log.data),
                block_hash:        Some(receipt.block_hash),
                block_number:      Some(receipt.block_number.into()),
                transaction_hash:  Some(receipt.tx_hash),
                transaction_index: Some(index.into()),
                log_index:         Some(log_idex.into()),
                removed:           false,
            };
            logs.push(web3_log);
        }
    }
}
