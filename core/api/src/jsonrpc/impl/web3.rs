use std::sync::Arc;

use ckb_types::core::cell::{CellProvider, CellStatus};
use ckb_types::prelude::Entity;
use jsonrpsee::core::Error;

use common_apm::metrics_rpc;
use protocol::traits::{APIAdapter, Context, Interoperation};
use protocol::types::{
    Block, BlockNumber, Bytes, CellDepWithPubKey, Hash, Hasher, Header, Hex, Proposal, Receipt,
    SignatureComponents, SignatureR, SignatureS, SignedTransaction, TxResp, UnverifiedTransaction,
    BASE_FEE_PER_GAS, H160, H256, MAX_FEE_HISTORY, MAX_RPC_GAS_CAP, MIN_TRANSACTION_GAS_LIMIT,
    U256,
};
use protocol::{
    async_trait, ckb_blake2b_256, codec::ProtocolCodec, lazy::PROTOCOL_VERSION, ProtocolResult,
};

use core_executor::system_contract::DataProvider;
use core_interoperation::utils::is_dummy_out_point;
use core_interoperation::InteroperationImpl;

use crate::jsonrpc::web3_types::{
    BlockCount, BlockId, FeeHistoryEmpty, FeeHistoryWithReward, FeeHistoryWithoutReward,
    RichTransactionOrHash, Web3Block, Web3CallRequest, Web3FeeHistory, Web3Filter, Web3Log,
    Web3Receipt, Web3Transaction,
};
use crate::jsonrpc::{error::RpcError, RpcResult, Web3RpcServer};
use crate::APIError;

pub(crate) const MAX_LOG_NUM: usize = 10000;

pub struct Web3RpcImpl<Adapter> {
    adapter:                    Arc<Adapter>,
    max_gas_cap:                U256,
    log_filter_max_block_range: u64,
}

impl<Adapter: APIAdapter> Web3RpcImpl<Adapter> {
    pub fn new(adapter: Arc<Adapter>, max_gas_cap: u64, log_filter_max_block_range: u64) -> Self {
        Self {
            adapter,
            max_gas_cap: max_gas_cap.into(),
            log_filter_max_block_range,
        }
    }

    async fn get_block_number_by_id(
        &self,
        block_id: Option<BlockId>,
    ) -> Result<Option<BlockNumber>, Error> {
        match block_id {
            Some(BlockId::Hash(ref hash)) => Ok(self
                .adapter
                .get_block_number_by_hash(Context::new(), *hash)
                .await
                .map_err(|e| Error::Custom(e.to_string()))?),
            _ => Ok(block_id.unwrap_or_default().into()),
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
                Proposal::new_without_state_root(&mock_header),
            )
            .await
    }

    async fn calculate_rewards(
        &self,
        block_number: u64,
        base_fee_par_gas: U256,
        txs: Vec<H256>,
        reward_percentiles: Vec<f64>,
        reward: &mut Vec<Vec<U256>>,
    ) -> Result<(), Error> {
        let receipts = self
            .adapter
            .get_receipts_by_hashes(Context::new(), block_number, &txs)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;

        let effective_priority_fees: Vec<U256> = receipts
            .iter()
            .map(|receipt| {
                receipt
                    .as_ref()
                    .map(|r| r.used_gas.saturating_sub(base_fee_par_gas))
                    .unwrap_or(U256::zero())
            })
            .collect();

        let reward_vec: Vec<U256> = reward_percentiles
            .iter()
            .map(|percentile| {
                let index =
                    calculate_effective_priority_fees_index(percentile, &effective_priority_fees);
                effective_priority_fees
                    .get(index)
                    .cloned()
                    .unwrap_or(U256::zero())
            })
            .collect();

        reward.push(reward_vec);

        Ok(())
    }

    async fn inner_fee_history(
        &self,
        height: Option<u64>,
        block_count: U256,
        reward_percentiles: &Option<Vec<f64>>,
    ) -> Result<(u64, Vec<U256>, Vec<f64>, Vec<Vec<U256>>), Error> {
        let latest_block = self
            .adapter
            .get_block_by_number(Context::new(), height)
            .await
            .map_err(|e| Error::Custom(e.to_string()))?
            .ok_or_else(|| Error::Custom("Latest block not found".to_string()))?;

        let latest_block_number = latest_block.header.number;
        let oldest_block_number = latest_block_number
            .saturating_sub(block_count.as_u64())
            .saturating_add(1);

        let mut bash_fee_per_gases: Vec<U256> = Vec::new();
        let mut gas_used_ratios: Vec<f64> = Vec::new();
        let mut reward: Vec<Vec<U256>> = Vec::new();

        for i in oldest_block_number..=latest_block_number {
            let block = match self
                .adapter
                .get_block_by_number(Context::new(), i.into())
                .await
            {
                Ok(Some(block)) => block,
                _ => continue,
            };

            let gas_used_ratio = calculate_gas_used_ratio(&block);
            gas_used_ratios.push(gas_used_ratio);
            bash_fee_per_gases.push(block.header.base_fee_per_gas);
            bash_fee_per_gases.push(next_block_base_fee_per_gas());

            if let Some(reward_percentiles) = reward_percentiles.clone() {
                let txs = block.tx_hashes;
                self.calculate_rewards(
                    block.header.number,
                    block.header.base_fee_per_gas,
                    txs,
                    reward_percentiles,
                    &mut reward,
                )
                .await?;
            }
        }

        Ok((
            oldest_block_number,
            bash_fee_per_gases,
            gas_used_ratios,
            reward,
        ))
    }

    async fn extract_interoperation_tx_sender(
        &self,
        utx: &UnverifiedTransaction,
        signature: &SignatureComponents,
    ) -> RpcResult<H160> {
        // Call CKB-VM mode
        if signature.r[0] == 0 {
            let r = rlp::decode::<CellDepWithPubKey>(&signature.r[1..])
                .map_err(|e| Error::Custom(e.to_string()))?;

            return Ok(Hasher::digest(&r.pub_key).into());
        }

        // Verify by CKB-VM mode
        let r = SignatureR::decode(&signature.r).map_err(|e| Error::Custom(e.to_string()))?;
        let s = SignatureS::decode(&signature.s).map_err(|e| Error::Custom(e.to_string()))?;
        let address_source = r.address_source();

        let ckb_tx_view =
            InteroperationImpl::dummy_transaction(r.clone(), s, Some(utx.signature_hash(true).0));
        let dummy_input = r.dummy_input();

        let input = ckb_tx_view
            .inputs()
            .get(address_source.index as usize)
            .ok_or(Error::Custom("Invalid address source".to_string()))?;

        log::debug!("[mempool]: verify interoperation tx sender \ntx view \n{:?}\ndummy input\n {:?}\naddress source\n{:?}\n", ckb_tx_view, dummy_input, address_source);

        // Dummy input mode
        if is_dummy_out_point(&input.previous_output()) {
            log::debug!("[mempool]: verify interoperation tx dummy input mode.");

            if let Some(cell) = dummy_input {
                if address_source.type_ == 1 && cell.type_script.is_none() {
                    return Err(Error::Custom(
                        "Invalid address source in dummy input mode".to_string(),
                    ));
                }

                let script_hash = if address_source.type_ == 0 {
                    cell.lock_script_hash()
                } else {
                    cell.type_script_hash().unwrap()
                };

                return Ok(Hasher::digest(script_hash).into());
            }

            return Err(Error::Custom("No dummy input cell".to_string()));
        }

        // Reality input mode
        let root = self
            .adapter
            .get_image_cell_root(Context::new())
            .await
            .map_err(|e| Error::Custom(e.to_string()))?;
        match DataProvider::new(root).cell(&input.previous_output(), true) {
            CellStatus::Live(cell) => {
                let script_hash = if address_source.type_ == 0 {
                    ckb_blake2b_256(cell.cell_output.lock().as_slice())
                } else if let Some(type_script) = cell.cell_output.type_().to_opt() {
                    ckb_blake2b_256(type_script.as_slice())
                } else {
                    return Err(Error::Custom("Invalid address source".to_string()));
                };

                Ok(Hasher::digest(script_hash).into())
            }
            _ => Err(Error::Custom("Cannot find input cell in ICSC".to_string())),
        }
    }
}

#[async_trait]
impl<Adapter: APIAdapter + 'static> Web3RpcServer for Web3RpcImpl<Adapter> {
    #[metrics_rpc("eth_sendRawTransaction")]
    async fn send_raw_transaction(&self, tx: Hex) -> RpcResult<H256> {
        let utx = UnverifiedTransaction::decode(&tx.as_bytes())
            .map_err(|e| Error::Custom(e.to_string()))?;

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

        if gas_limit > self.max_gas_cap {
            return Err(Error::Custom(
                "The transaction gas limit is too large".to_string(),
            ));
        }

        utx.check_hash().map_err(|e| Error::Custom(e.to_string()))?;

        let interoperation_sender = if let Some(sig) = utx.signature.as_ref() {
            if sig.is_eth_sig() {
                None
            } else {
                Some(self.extract_interoperation_tx_sender(&utx, sig).await?)
            }
        } else {
            return Err(Error::Custom("The transaction is not signed".to_string()));
        };

        let stx = SignedTransaction::from_unverified(utx, interoperation_sender)
            .map_err(|e| Error::Custom(e.to_string()))?;
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
        block_id: Option<BlockId>,
    ) -> RpcResult<U256> {
        match block_id.unwrap_or_default() {
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
            BlockId::Hash(ref hash) => {
                let number = self
                    .adapter
                    .get_block_number_by_hash(Context::new(), *hash)
                    .await
                    .map_err(|e| Error::Custom(e.to_string()))?;
                Ok(self
                    .adapter
                    .get_account(Context::new(), address, number)
                    .await
                    .map(|account| account.nonce)
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
    async fn get_balance(&self, address: H160, block_id: Option<BlockId>) -> RpcResult<U256> {
        let number = self.get_block_number_by_id(block_id).await?;

        Ok(self
            .adapter
            .get_account(Context::new(), address, number)
            .await
            .map_or(U256::zero(), |account| account.balance))
    }

    #[metrics_rpc("eth_call")]
    async fn call(&self, req: Web3CallRequest, block_id: Option<BlockId>) -> RpcResult<Hex> {
        if req.gas_price.unwrap_or_default() > U256::from(u64::MAX) {
            return Err(Error::Custom("The gas price is too large".to_string()));
        }

        if req.gas.unwrap_or_default() > U256::from(MAX_RPC_GAS_CAP) {
            return Err(Error::Custom("The gas limit is too large".to_string()));
        }

        let number = self.get_block_number_by_id(block_id).await?;

        let data_bytes = req
            .data
            .as_ref()
            .map(|hex| hex.as_bytes())
            .unwrap_or_default();
        let resp = self
            .call_evm(req, data_bytes, number)
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
            Some(BlockId::Num(n)) => Some(n.as_u64()),
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
    async fn get_code(&self, address: H160, block_id: Option<BlockId>) -> RpcResult<Hex> {
        let number = self.get_block_number_by_id(block_id).await?;

        let account = self
            .adapter
            .get_account(Context::new(), address, number)
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
                    from_receipt_to_web3_log(
                        index,
                        topics,
                        address.as_ref().unwrap_or(&&Vec::new()),
                        &receipt,
                        logs,
                    );

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
                            BlockId::Num(n) => n.as_u64(),
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

                if end.saturating_sub(start) > self.log_filter_max_block_range {
                    return Err(Error::Custom(format!(
                        "Invalid block range {:?} to {:?}, limit to {:?}",
                        start, end, self.log_filter_max_block_range
                    )));
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
        block_count: BlockCount,
        newest_block: BlockId,
        reward_percentiles: Option<Vec<f64>>,
    ) -> RpcResult<Web3FeeHistory> {
        check_reward_percentiles(&reward_percentiles)?;

        let mut blocks_count;
        match block_count {
            BlockCount::U256Type(n) => blocks_count = n,
            BlockCount::U64Type(n) => blocks_count = n.into(),
        }
        // Between 1 and 1024 blocks can be requested in a single query.
        if blocks_count > MAX_FEE_HISTORY.into() {
            blocks_count = MAX_FEE_HISTORY.into();
        }
        if blocks_count == 0.into() {
            return Ok(Web3FeeHistory::ZeroBlockCount(FeeHistoryEmpty {
                oldest_block:   U256::zero(),
                gas_used_ratio: None,
            }));
        }
        match newest_block {
            BlockId::Num(number) => {
                let (oldest_block_number, bash_fee_per_gases, gas_used_ratios, reward) = self
                    .inner_fee_history(number.as_u64().into(), blocks_count, &reward_percentiles)
                    .await?;

                match reward_percentiles {
                    None => Ok(Web3FeeHistory::WithoutReward(FeeHistoryWithoutReward {
                        oldest_block:     oldest_block_number.into(),
                        base_fee_per_gas: bash_fee_per_gases,
                        gas_used_ratio:   gas_used_ratios,
                    })),
                    Some(reward_percentiles) => {
                        if reward_percentiles.is_empty() {
                            return Ok(Web3FeeHistory::WithoutReward(FeeHistoryWithoutReward {
                                oldest_block:     oldest_block_number.into(),
                                base_fee_per_gas: bash_fee_per_gases,
                                gas_used_ratio:   gas_used_ratios,
                            }));
                        }
                        Ok(Web3FeeHistory::WithReward(FeeHistoryWithReward {
                            oldest_block: oldest_block_number.into(),
                            reward,
                            base_fee_per_gas: bash_fee_per_gases,
                            gas_used_ratio: gas_used_ratios,
                        }))
                    }
                }
            }
            BlockId::Latest | BlockId::Pending => {
                let (oldest_block_number, bash_fee_per_gases, gas_used_ratios, reward) = self
                    .inner_fee_history(None, blocks_count, &reward_percentiles)
                    .await?;

                match reward_percentiles {
                    None => Ok(Web3FeeHistory::WithoutReward(FeeHistoryWithoutReward {
                        oldest_block:     oldest_block_number.into(),
                        base_fee_per_gas: bash_fee_per_gases,
                        gas_used_ratio:   gas_used_ratios,
                    })),
                    Some(reward_percentiles) => {
                        if reward_percentiles.is_empty() {
                            return Ok(Web3FeeHistory::WithoutReward(FeeHistoryWithoutReward {
                                oldest_block:     oldest_block_number.into(),
                                base_fee_per_gas: bash_fee_per_gases,
                                gas_used_ratio:   gas_used_ratios,
                            }));
                        }
                        Ok(Web3FeeHistory::WithReward(FeeHistoryWithReward {
                            oldest_block: oldest_block_number.into(),
                            reward,
                            base_fee_per_gas: bash_fee_per_gases,
                            gas_used_ratio: gas_used_ratios,
                        }))
                    }
                }
            }
            BlockId::Earliest => {
                let first_block = self
                    .adapter
                    .get_block_by_number(Context::new(), Some(0))
                    .await
                    .map_err(|e| Error::Custom(e.to_string()))?
                    .unwrap();
                let base_fee_per_gas = vec![
                    first_block.header.base_fee_per_gas,
                    next_block_base_fee_per_gas(),
                ];
                let gas_used_ratio = vec![calculate_gas_used_ratio(&first_block)];

                match reward_percentiles {
                    None => Ok(Web3FeeHistory::WithoutReward(FeeHistoryWithoutReward {
                        oldest_block: first_block.header.number.into(),
                        base_fee_per_gas,
                        gas_used_ratio,
                    })),
                    Some(reward_percentiles) => {
                        if reward_percentiles.is_empty() {
                            return Ok(Web3FeeHistory::WithoutReward(FeeHistoryWithoutReward {
                                oldest_block: first_block.header.number.into(),
                                base_fee_per_gas,
                                gas_used_ratio,
                            }));
                        }
                        let mut reward: Vec<Vec<U256>> = Vec::new();
                        self.calculate_rewards(
                            first_block.header.number,
                            first_block.header.base_fee_per_gas,
                            first_block.tx_hashes,
                            reward_percentiles,
                            &mut reward,
                        )
                        .await?;
                        Ok(Web3FeeHistory::WithReward(FeeHistoryWithReward {
                            oldest_block: first_block.header.number.into(),
                            reward,
                            base_fee_per_gas,
                            gas_used_ratio,
                        }))
                    }
                }
            }
            _ => {
                return Err(Error::Custom(format!(
                    "Invalid parameter newest_block {:?}",
                    newest_block
                )))
            }
        }
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
        block_id: Option<BlockId>,
    ) -> RpcResult<Hex> {
        let number = self.get_block_number_by_id(block_id).await?;

        let block = self
            .adapter
            .get_block_by_number(Context::new(), number)
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

    #[metrics_rpc("eth_protocolVersion")]
    async fn protocol_version(&self) -> RpcResult<Hex> {
        Ok((**PROTOCOL_VERSION.load()).clone())
    }

    #[metrics_rpc("eth_getUncleByBlockHashAndIndex")]
    async fn get_uncle_by_block_hash_and_index(
        &self,
        _hash: Hash,
        _index: U256,
    ) -> RpcResult<Option<Web3Block>> {
        Ok(None)
    }

    #[metrics_rpc("eth_getUncleByBlockNumberAndIndex")]
    async fn get_uncle_by_block_number_and_index(
        &self,
        _number: BlockId,
        _index: U256,
    ) -> RpcResult<Option<Web3Block>> {
        Ok(None)
    }

    #[metrics_rpc("eth_getUncleCountByBlockHash")]
    async fn get_uncle_count_by_block_hash(&self, _hash: Hash) -> RpcResult<U256> {
        Ok(U256::zero())
    }

    #[metrics_rpc("eth_getUncleCountByBlockNumber")]
    async fn get_uncle_count_by_block_number(&self, _number: BlockId) -> RpcResult<U256> {
        Ok(U256::zero())
    }
}

// 1. checks for rewardPercentile's sorted-ness
// 2. check if any of rewardPercentile is greater than 100 or less than 0
fn check_reward_percentiles(reward_percentiles: &Option<Vec<f64>>) -> RpcResult<()> {
    reward_percentiles
        .as_ref()
        .and_then(|percentiles| {
            percentiles
                .windows(2)
                .enumerate()
                .find(|(_, window)| window[0] >= window[1] || window[0] < 0.0 || window[0] > 100.0)
                .map(|(_, vec)| vec)
        })
        .map(|vec| {
            Err(Error::Custom(format!(
                "Invalid parameter reward_percentiles: {} {}",
                vec[0], vec[1]
            )))
        })
        .unwrap_or(Ok(()))
}

// Calculates the gas used ratio for the block.
fn calculate_gas_used_ratio(block: &Block) -> f64 {
    (block.header.gas_limit != U256::zero())
        .then(|| {
            block.header.gas_used.as_u64() as f64 / block.header.gas_limit.as_u64() as f64 * 100f64
        })
        .unwrap_or(0f64)
}

// Calculates the effective priority fees index in the effective priority fees
// vector.
fn calculate_effective_priority_fees_index(
    percentile: &f64,
    effective_priority_fees: &Vec<U256>,
) -> usize {
    ((percentile * effective_priority_fees.len() as f64 / 100f64).floor() as usize)
        .saturating_sub(1)
}

// Get the base fee per gas for the next block.
fn next_block_base_fee_per_gas() -> U256 {
    BASE_FEE_PER_GAS.into()
}

fn mock_header_by_call_req(latest_header: Header, call_req: &Web3CallRequest) -> Header {
    Header {
        version:                  latest_header.version,
        prev_hash:                latest_header.prev_hash,
        proposer:                 latest_header.proposer,
        state_root:               latest_header.state_root,
        transactions_root:        Default::default(),
        signed_txs_hash:          Default::default(),
        receipts_root:            Default::default(),
        log_bloom:                Default::default(),
        timestamp:                latest_header.timestamp,
        number:                   latest_header.number,
        gas_used:                 latest_header.gas_used,
        gas_limit:                if let Some(gas_limit) = call_req.gas {
            gas_limit
        } else {
            latest_header.gas_limit
        },
        extra_data:               Default::default(),
        base_fee_per_gas:         if let Some(base_fee) = call_req.max_fee_per_gas {
            base_fee
        } else {
            latest_header.base_fee_per_gas
        },
        proof:                    latest_header.proof,
        call_system_script_count: 0,
        chain_id:                 latest_header.chain_id,
    }
}

pub fn from_receipt_to_web3_log(
    index: usize,
    topics: &[Option<Vec<Option<Hash>>>],
    address: &[H160],
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
        if (address.is_empty() || address.contains(&log.address)) && contains_topic!(topics, log) {
            let web3_log = Web3Log {
                address:           log.address,
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
