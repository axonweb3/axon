use std::collections::{BTreeSet, VecDeque};
use std::sync::Arc;

use async_std::task::block_on;
use jsonrpsee::core::Error;
use parking_lot::Mutex;

use common_metrics_derive::metrics_rpc;
use core_consensus::SYNC_STATUS;
use protocol::traits::{APIAdapter, Context};
use protocol::types::{
    Block, BlockNumber, Bytes, Hash, Hasher, Header, Hex, Receipt, SignedTransaction, TxResp,
    UnverifiedTransaction, H160, H256, H64, U256,
};
use protocol::{async_trait, codec::ProtocolCodec, ProtocolResult};

use crate::jsonrpc::poll_filter::{PollFilter, SyncPollFilter};
use crate::jsonrpc::poll_manager::PollManager;

use crate::jsonrpc::web3_types::{
    BlockId, ChangeWeb3Filter, Filter, FilterChanges, Index, RichTransactionOrHash, WEB3Work,
    Web3Block, Web3CallRequest, Web3FeeHistory, Web3Filter, Web3Log, Web3Receipt, Web3SyncStatus,
    Web3Transaction,
};

use crate::jsonrpc::{AxonJsonRpcServer, RpcResult};
use crate::APIError;

pub struct JsonRpcImpl<Adapter> {
    adapter: Arc<Adapter>,
    version: String,
    polls:   Mutex<PollManager<SyncPollFilter>>,
}

impl<Adapter: APIAdapter> JsonRpcImpl<Adapter> {
    pub fn new(adapter: Arc<Adapter>, version: &str, poll_lifetime: u32) -> Self {
        Self {
            adapter,
            version: version.to_string(),
            polls: Mutex::new(PollManager::new(poll_lifetime)),
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

    fn polls(&self) -> &Mutex<PollManager<SyncPollFilter>> {
        &self.polls
    }

    fn get_block_number_by_hash(&self, hash: Hash) -> ProtocolResult<u64> {
        // fixme: how can i call a async method as below witch is
        // self.adapter.get_number_by_hash.
        let ret_number = block_on(self.adapter.get_number_by_hash(Context::new(), hash))?
            .ok_or_else(|| {
                APIError::Storage(format!("cannot get {:?} number by block hash.", hash))
            })?;
        Ok(ret_number)
    }

    fn convert_block_hash(&self, block_id: BlockId) -> Option<Hash> {
        match block_id {
            BlockId::Hash(hash) => Some(hash),
            BlockId::Num(n) => {
                let mut block_hash: Option<Hash> = None;
                // fixme: how can i call a async method as below witch is
                // self.adapter.get_block_by_number.
                let ret_block = block_on(self.adapter.get_block_by_number(Context::new(), Some(n)));
                if let Ok(Some(block)) = ret_block {
                    block_hash = Some(block.header.proof.block_hash);
                }
                block_hash
            }
            // BlockId::Earliest => self.numbers.read().get(&0).cloned(),
            BlockId::Latest => {
                let mut block_hash: Option<Hash> = None;
                // fixme: how can i call a async method as below witch is
                // self.adapter.get_block_by_number.
                let ret_block = block_on(self.adapter.get_block_by_number(Context::new(), None));
                if let Ok(Some(block)) = ret_block {
                    block_hash = Some(block.header.proof.block_hash);
                }
                block_hash
            }
        }
    }

    fn convert_block_number(&self, block_id: BlockId) -> Option<u64> {
        match block_id {
            BlockId::Hash(hash) => {
                let mut block_number: Option<u64> = None;
                // fixme: how can i call a async method as below witch is
                // self.adapter.get_block_by_hash.
                let ret_block = block_on(self.adapter.get_block_by_hash(Context::new(), hash));
                if let Ok(Some(block)) = ret_block {
                    block_number = Some(block.header.number);
                }
                block_number
            }
            BlockId::Num(n) => Some(n),
            // BlockId::Earliest => self.numbers.read().get(&0).cloned(),
            BlockId::Latest => {
                let mut block_number: Option<u64> = None;
                let ret_block = block_on(self.adapter.get_block_by_number(Context::new(), None));
                if let Ok(Some(block)) = ret_block {
                    block_number = Some(block.header.number);
                }
                block_number
            }
        }
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

    #[metrics_rpc("eth_mining")]
    async fn mining(&self) -> RpcResult<bool> {
        Ok(false)
    }

    #[metrics_rpc("net_peerCount")]
    async fn peer_count(&self) -> RpcResult<U256> {
        self.adapter
            .peer_count(Context::new())
            .await
            .map_err(|e| Error::Custom(e.to_string()))
    }

    #[metrics_rpc("eth_syncing")]
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
                            BlockId::Hash(hash) => {
                                let ret_num = self.get_block_number_by_hash(hash);
                                match ret_num {
                                    Ok(num) => num,
                                    _ => 0u64,
                                }
                            }
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

    #[metrics_rpc("eth_feeHistory")]
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

    #[metrics_rpc("web3_clientVersion")]
    async fn client_version(&self) -> RpcResult<String> {
        Ok(self.version.clone())
    }

    #[metrics_rpc("eth_accounts")]
    async fn accounts(&self) -> RpcResult<Vec<Hex>> {
        Ok(vec![])
    }

    #[metrics_rpc("web3_sha3")]
    async fn sha3(&self, data: Hex) -> RpcResult<Hash> {
        let decode_data =
            Hex::decode(data.as_string()).map_err(|e| Error::Custom(e.to_string()))?;
        let ret = Hasher::digest(decode_data.as_ref());
        Ok(ret)
    }

    #[metrics_rpc("eth_getTransactionCountByHash")]
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
        number: BlockId,
    ) -> RpcResult<Hex> {
        let block = self
            .adapter
            .get_block_by_number(Context::new(), number.into())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .ok_or_else(|| Error::Custom("Can't find this block".to_string()))?;
        let value = self
            .adapter
            .get_storage_at(Context::new(), address, position, block.header.state_root)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        Ok(Hex::encode(&value))
    }

    async fn coinbase(&self) -> RpcResult<H160> {
        // fixme: how to get the the coinbase value
        Ok(H160::default())
    }

    async fn hashrate(&self) -> RpcResult<U256> {
        Ok(U256::from(1u64))
    }

    async fn get_work(&self) -> RpcResult<(Hash, Hash, Hash)> {
        let work = WEB3Work {
            pow_hash:  H256::default(), // how to get the pow_hash
            seed_hash: H256::default(),
            target:    H256::default(),
        };
        Ok((work.pow_hash, work.pow_hash, work.target))
    }

    async fn submit_work(&self, _nc: U256, _hash: H256, _summary: Hex) -> RpcResult<bool> {
        Ok(true)
    }

    async fn submit_hashrate(&self, _hash_rate: Hex, _client_id: Hex) -> RpcResult<bool> {
        Ok(true)
    }

    async fn new_filter(&self, filter: ChangeWeb3Filter) -> RpcResult<U256> {
        let mut polls = self.polls.lock();
        let block_number = best_block_number();
        let include_pending = false;
        let filter = filter.try_into();
        let id = polls.create_poll(SyncPollFilter::new(PollFilter::Logs {
            block_number,
            filter,
            include_pending,
            last_block_hash: None,
            previous_logs: Default::default(),
        }));
        Ok(id.into())
    }

    async fn new_block_filter(&self) -> RpcResult<U256> {
        let mut polls = self.polls.lock();
        // +1, since we don't want to include the current block
        let id = polls.create_poll(SyncPollFilter::new(PollFilter::Block {
            last_block_number:      best_block_number(),
            recent_reported_hashes: VecDeque::with_capacity(PollFilter::max_block_history_size()),
        }));
        Ok(id.into())
    }

    async fn new_pending_transaction_filter(&self) -> RpcResult<U256> {
        let mut polls = self.polls.lock();
        let pending_transactions = pending_transaction_hashes();
        let id = polls.create_poll(SyncPollFilter::new(PollFilter::PendingTransaction(
            pending_transactions,
        )));
        Ok(id.into())
    }

    async fn filter_changes(&self, index: Index) -> RpcResult<FilterChanges> {
        let filter = match self.polls().lock().poll_mut(&index.value()) {
            Some(filter) => filter.clone(),
            None => return Err(Error::Custom("can not find filter".to_string())),
        };

        let ret_filter_changes = filter.modify(|filter| match *filter {
            PollFilter::Block {
                ref mut last_block_number,
                ref mut recent_reported_hashes,
            } => {
                // Check validity of recently reported blocks -- in case of re-org, rewind block
                // to last valid
                while let Some((num, hash)) = recent_reported_hashes.front().cloned() {
                    if self.convert_block_hash(BlockId::Num(num)) == Some(hash) {
                        break;
                    }
                    *last_block_number = num - 1;
                    recent_reported_hashes.pop_front();
                }
                let current_number = best_block_number();
                let mut hashes = Vec::new();
                for n in (*last_block_number + 1)..=current_number {
                    let block_number = BlockId::Num(n);
                    if let Some(hash) = self.convert_block_hash(block_number) {
                        *last_block_number = n;
                        hashes.push(hash);
                        // Only keep the most recent history
                        if recent_reported_hashes.len() >= PollFilter::max_block_history_size() {
                            recent_reported_hashes.pop_back();
                        }
                        recent_reported_hashes.push_front((n, hash));
                    }
                }

                FilterChanges::Hashes(hashes)
            }
            PollFilter::PendingTransaction(ref mut previous_hashes) => {
                // get hashes of pending transactions
                let current_hashes = pending_transaction_hashes();

                let new_hashes = {
                    // find all new hashes
                    current_hashes
                        .difference(previous_hashes)
                        .cloned()
                        .map(Into::into)
                        .collect()
                };

                // save all hashes of pending transactions
                *previous_hashes = current_hashes;

                // return new hashes
                FilterChanges::Hashes(new_hashes)
            }
            PollFilter::Logs {
                ref mut block_number,
                ref mut last_block_hash,
                previous_logs: _,
                ref filter,
                include_pending: _,
            } => {
                // retrive the current block number
                let current_number = best_block_number();

                let mut filter = filter.clone();

                let (reorg, reorg_len) = last_block_hash.map_or_else(
                    || (Vec::new(), 0),
                    |h| {
                        // fixme: how can i call a async method as below witch is self.removed_logs.
                        let ret = block_on(self.removed_logs(h, filter.clone()));
                        match ret {
                            Ok(inter_ret) => inter_ret,
                            _ => {
                                let empty_logs: Vec<Web3Log> = vec![];
                                (empty_logs, 0u64)
                            }
                        }
                    },
                );
                *block_number -= reorg_len as u64;

                filter.from_block = BlockId::Num(*block_number);
                filter.to_block = BlockId::Num(3u64); // BlockId::Latest;

                // save the number of the next block as a first block from which
                // we want to get logs
                *block_number = current_number + 1;

                // save the current block hash, which we used to get back to the
                // canon chain in case of reorg.
                *last_block_hash = self.convert_block_hash(BlockId::Num(current_number));

                // retrieve logs in range from_block..min(BlockId::Latest..to_block)
                let limit = filter.limit;
                let mut web3_logs: Vec<Web3Log> = vec![];

                for topic in filter.topics {
                    if let Some(addrs) = filter.address.clone() {
                        for addr in addrs {
                            let logs = block_on(self.get_logs(Web3Filter {
                                from_block: Some(filter.from_block.clone()),
                                to_block:   Some(filter.to_block.clone()),
                                block_hash: None,
                                address:    Some(addr),
                                topics:     topic.clone(),
                                limit:      None,
                            }));

                            if let Ok(ret) = logs {
                                for log in ret {
                                    web3_logs.push(log)
                                }
                            }
                        }
                    } else {
                        let logs = block_on(self.get_logs(Web3Filter {
                            from_block: Some(filter.from_block.clone()),
                            to_block:   Some(filter.to_block.clone()),
                            block_hash: None,
                            address:    None,
                            topics:     topic.clone(),
                            limit:      None,
                        }));

                        if let Ok(ret) = logs {
                            for log in ret {
                                web3_logs.push(log)
                            }
                        }
                    }
                }
                // append reorg logs in the front
                web3_logs.extend(reorg);
                let limit_logs = limit_logs(web3_logs, limit);
                FilterChanges::Logs(limit_logs)
            }
        });
        Ok(ret_filter_changes)
    }

    async fn removed_logs(
        &self,
        block_hash: H256,
        web3_filter: Filter,
    ) -> RpcResult<(Vec<Web3Log>, u64)> {
        let filter = &web3_filter;

        let inner = || -> Option<Vec<H256>> {
            let mut route = Vec::new();
            let mut current_block_hash = block_hash;
            let current_block_header = block_on(self.adapter.get_block_header_by_number(
                Context::new(),
                self.convert_block_number(BlockId::Hash(current_block_hash)),
            ));

            if let Ok(Some(hea)) = current_block_header {
                while Some(current_block_hash) != self.convert_block_hash(BlockId::Num(hea.number))
                {
                    route.push(current_block_hash);

                    current_block_hash = hea.prev_hash;
                }
            }

            Some(route)
        };

        let route = inner().unwrap_or_default();
        let mut web3_logs: Vec<Web3Log> = vec![];
        let route_len = route.len() as u64;
        for block_hash in route.into_iter() {
            let mut filter = filter.clone();
            filter.from_block = BlockId::Hash(block_hash);
            filter.to_block = filter.from_block.clone();

            let logs = block_on(self.get_logs(Web3Filter {
                from_block: Some(filter.from_block.clone()),
                to_block:   Some(filter.to_block.clone()),
                block_hash: None,
                address:    None,
                topics:     None,
                limit:      None,
            }));

            if let Ok(ret) = logs {
                for log in ret {
                    let mut log: Web3Log = log;
                    log.log_type = "removed".to_string();
                    log.removed = true;
                    web3_logs.push(log);
                }
            }
        }

        Ok((web3_logs, route_len))
    }

    async fn uninstall_filter(&self, idx: Index) -> RpcResult<bool> {
        Ok(self.polls.lock().remove_poll(&idx.value()))
    }
}

fn best_block_number() -> u64 {
    0u64 // fixme:: axon how to get the best_block
}

fn pending_transaction_hashes() -> BTreeSet<H256> {
    // axon haven't pending status
    let btree: BTreeSet<H256> = BTreeSet::new();
    btree
}

fn limit_logs(mut logs: Vec<Web3Log>, limit: Option<usize>) -> Vec<Web3Log> {
    let len = logs.len();
    match limit {
        Some(limit) if len >= limit => logs.split_off(len - limit),
        _ => logs,
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
                    log_type:          "".to_string(),
                };
                logs.push(web3_log);
            }
        }
    }
}
