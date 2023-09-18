mod error;
mod r#impl;
mod web3_types;
mod ws_subscription;

use std::{collections::HashMap, sync::Arc};

use ckb_jsonrpc_types::{CellInfo, HeaderView as CkbHeaderView, OutPoint};
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

use common_config_parser::types::{spec::HardforkName, Config};
use protocol::traits::APIAdapter;
use protocol::types::{
    Block, CkbRelatedInfo, Hash, Hex, Metadata, Proof, Proposal, H160, H256, U256,
};
use protocol::ProtocolResult;

use crate::jsonrpc::web3_types::{
    BlockId, FilterChanges, HardforkStatus, RawLoggerFilter, Web3Block, Web3CallRequest,
    Web3FeeHistory, Web3Filter, Web3Log, Web3Receipt, Web3SyncStatus, Web3Transaction,
};
use crate::jsonrpc::ws_subscription::{ws_subscription_module, HexIdProvider};
use crate::APIError;

use self::web3_types::BlockCount;

#[rpc(server)]
pub trait Web3Rpc {
    /// Sends signed transaction, returning its hash.
    #[method(name = "eth_sendRawTransaction")]
    async fn send_raw_transaction(&self, tx: Hex) -> RpcResult<H256>;

    /// Get transaction by its hash.
    #[method(name = "eth_getTransactionByHash")]
    async fn get_transaction_by_hash(&self, hash: H256) -> RpcResult<Option<Web3Transaction>>;

    /// Returns block with given number.
    #[method(name = "eth_getBlockByNumber")]
    async fn get_block_by_number(
        &self,
        number: BlockId,
        show_rich_tx: bool,
    ) -> RpcResult<Option<Web3Block>>;

    #[method(name = "eth_getBlockByHash")]
    async fn get_block_by_hash(
        &self,
        hash: H256,
        show_rich_tx: bool,
    ) -> RpcResult<Option<Web3Block>>;

    #[method(name = "eth_blockNumber")]
    async fn block_number(&self) -> RpcResult<U256>;

    #[method(name = "eth_getTransactionCount")]
    async fn get_transaction_count(
        &self,
        address: H160,
        number: Option<BlockId>,
    ) -> RpcResult<U256>;

    #[method(name = "eth_getBlockTransactionCountByNumber")]
    async fn get_block_transaction_count_by_number(&self, number: BlockId) -> RpcResult<U256>;

    #[method(name = "eth_getBalance")]
    async fn get_balance(&self, address: H160, number: Option<BlockId>) -> RpcResult<U256>;

    #[method(name = "eth_call")]
    async fn call(&self, req: Web3CallRequest, number: Option<BlockId>) -> RpcResult<Hex>;

    #[method(name = "eth_estimateGas")]
    async fn estimate_gas(&self, req: Web3CallRequest, number: Option<BlockId>) -> RpcResult<U256>;

    #[method(name = "eth_getCode")]
    async fn get_code(&self, address: H160, number: Option<BlockId>) -> RpcResult<Hex>;

    #[method(name = "eth_getTransactionReceipt")]
    async fn get_transaction_receipt(&self, hash: H256) -> RpcResult<Option<Web3Receipt>>;

    #[method(name = "eth_gasPrice")]
    async fn gas_price(&self) -> RpcResult<U256>;

    #[method(name = "eth_getLogs")]
    async fn get_logs(&self, filter: Web3Filter) -> RpcResult<Vec<Web3Log>>;

    #[method(name = "eth_feeHistory")]
    async fn fee_history(
        &self,
        block_count: BlockCount,
        newest_block: BlockId,
        reward_percentiles: Option<Vec<f64>>,
    ) -> RpcResult<Web3FeeHistory>;

    #[method(name = "eth_accounts")]
    async fn accounts(&self) -> RpcResult<Vec<Hex>>;

    #[method(name = "eth_getBlockTransactionCountByHash")]
    async fn get_block_transaction_count_by_hash(&self, hash: Hash) -> RpcResult<U256>;

    #[method(name = "eth_getTransactionByBlockHashAndIndex")]
    async fn get_transaction_by_block_hash_and_index(
        &self,
        hash: Hash,
        position: U256,
    ) -> RpcResult<Option<Web3Transaction>>;

    #[method(name = "eth_getTransactionByBlockNumberAndIndex")]
    async fn get_transaction_by_block_number_and_index(
        &self,
        number: BlockId,
        position: U256,
    ) -> RpcResult<Option<Web3Transaction>>;

    #[method(name = "net_peerCount")]
    async fn peer_count(&self) -> RpcResult<U256>;

    #[method(name = "eth_getStorageAt")]
    async fn get_storage_at(
        &self,
        address: H160,
        position: U256,
        number: Option<BlockId>,
    ) -> RpcResult<Hex>;

    #[method(name = "eth_protocolVersion")]
    async fn protocol_version(&self) -> RpcResult<Hex>;

    #[method(name = "eth_getUncleByBlockHashAndIndex")]
    async fn get_uncle_by_block_hash_and_index(
        &self,
        hash: Hash,
        position: U256,
    ) -> RpcResult<Option<Web3Block>>;

    #[method(name = "eth_getUncleByBlockNumberAndIndex")]
    async fn get_uncle_by_block_number_and_index(
        &self,
        number: BlockId,
        position: U256,
    ) -> RpcResult<Option<Web3Block>>;

    #[method(name = "eth_getUncleCountByBlockHash")]
    async fn get_uncle_count_by_block_hash(&self, hash: Hash) -> RpcResult<U256>;

    #[method(name = "eth_getUncleCountByBlockNumber")]
    async fn get_uncle_count_by_block_number(&self, number: BlockId) -> RpcResult<U256>;
}

#[rpc(server)]
pub trait Web3Filter {
    #[method(name = "eth_newFilter")]
    async fn new_filter(&self, filter: RawLoggerFilter) -> RpcResult<U256>;

    #[method(name = "eth_newBlockFilter")]
    async fn block_filter(&self) -> RpcResult<U256>;

    #[method(name = "eth_getFilterLogs")]
    async fn get_filter_logs(&self, id: U256) -> RpcResult<FilterChanges>;

    #[method(name = "eth_getFilterChanges")]
    async fn get_filter_changes(&self, id: U256) -> RpcResult<FilterChanges>;

    #[method(name = "eth_uninstallFilter")]
    async fn uninstall_filter(&self, id: U256) -> RpcResult<bool>;
}

#[rpc(server)]
pub trait AxonNodeRpc {
    #[method(name = "eth_chainId")]
    fn chain_id(&self) -> RpcResult<U256>;

    #[method(name = "net_version")]
    fn net_version(&self) -> RpcResult<String>;

    #[method(name = "web3_clientVersion")]
    fn client_version(&self) -> RpcResult<String>;

    #[method(name = "net_listening")]
    fn listening(&self) -> RpcResult<bool>;

    #[method(name = "eth_syncing")]
    fn syncing(&self) -> RpcResult<Web3SyncStatus>;

    #[method(name = "eth_mining")]
    fn mining(&self) -> RpcResult<bool>;

    #[method(name = "eth_coinbase")]
    fn coinbase(&self) -> RpcResult<H160>;

    #[method(name = "eth_hashrate")]
    fn hashrate(&self) -> RpcResult<U256>;

    #[method(name = "eth_submitWork ")]
    fn submit_work(&self, _nc: U256, _hash: H256, _summary: Hex) -> RpcResult<bool>;

    #[method(name = "eth_submitHashrate")]
    fn submit_hashrate(&self, _hash_rate: Hex, _client_id: Hex) -> RpcResult<bool>;

    #[method(name = "web3_sha3")]
    fn sha3(&self, data: Hex) -> RpcResult<Hash>;

    #[method(name = "pprof")]
    fn pprof(&self, enable: bool) -> RpcResult<bool>;
}

#[rpc(server)]
pub trait AxonRpc {
    #[method(name = "axon_getBlockById")]
    async fn get_block_by_id(&self, block_id: BlockId) -> RpcResult<Option<Block>>;

    #[method(name = "axon_getProofById")]
    async fn get_proof_by_id(&self, block_id: BlockId) -> RpcResult<Option<Proof>>;

    #[method(name = "axon_getMetadataByNumber")]
    async fn get_metadata_by_number(&self, block_number: U256) -> RpcResult<Metadata>;

    #[method(name = "axon_getProposalByNumber")]
    async fn get_proposal_by_number(&self, block_number: U256) -> RpcResult<Proposal>;

    #[method(name = "axon_getCurrentMetadata")]
    async fn get_current_metadata(&self) -> RpcResult<Metadata>;

    #[method(name = "axon_getCkbRelatedInfo")]
    async fn get_ckb_related_info(&self) -> RpcResult<CkbRelatedInfo>;

    #[method(name = "axon_getHardforkInfo")]
    async fn hardfork_infos(&self) -> RpcResult<HashMap<HardforkName, HardforkStatus>>;
}

#[rpc(server)]
pub trait CkbLightClientRpc {
    #[method(name = "ckb_getBlockHeaderByHash")]
    async fn get_block_header_by_hash(&self, hash: Hash) -> RpcResult<Option<CkbHeaderView>>;

    #[method(name = "ckb_getLiveCell")]
    async fn get_cell_info(
        &self,
        out_point: OutPoint,
        with_data: bool,
    ) -> RpcResult<Option<CellInfo>>;
}

pub async fn run_jsonrpc_server<Adapter: APIAdapter + 'static>(
    version: String,
    config: Config,
    adapter: Arc<Adapter>,
) -> ProtocolResult<(Option<ServerHandle>, Option<ServerHandle>)> {
    let mut ret = (None, None);

    let mut rpc = r#impl::Web3RpcImpl::new(
        Arc::clone(&adapter),
        config.web3.max_gas_cap,
        config.web3.log_filter_max_block_range,
    )
    .into_rpc();

    let node_rpc = r#impl::NodeRpcImpl::new(version, config.data_path).into_rpc();
    let axon_rpc = r#impl::AxonRpcImpl::new(Arc::clone(&adapter)).into_rpc();
    let filter =
        r#impl::filter_module(Arc::clone(&adapter), config.web3.log_filter_max_block_range)
            .into_rpc();
    let ckb_light_client_rpc = r#impl::CkbLightClientRpcImpl::new(Arc::clone(&adapter)).into_rpc();

    rpc.merge(node_rpc).unwrap();
    rpc.merge(axon_rpc).unwrap();
    rpc.merge(filter).unwrap();
    rpc.merge(ckb_light_client_rpc).unwrap();

    if let Some(addr) = config.rpc.http_listening_address {
        let server = ServerBuilder::new()
            .http_only()
            .max_request_body_size(config.rpc.max_payload_size)
            .max_response_body_size(config.rpc.max_payload_size)
            .max_connections(config.rpc.maxconn)
            .build(addr)
            .await
            .map_err(|e| APIError::HttpServer(e.to_string()))?;

        ret.0 = Some(server.start(rpc.clone()));
    }

    if let Some(addr) = config.rpc.ws_listening_address {
        let server = ServerBuilder::new()
            .ws_only()
            .max_request_body_size(config.rpc.max_payload_size)
            .max_request_body_size(config.rpc.max_payload_size)
            .max_connections(config.rpc.maxconn)
            .set_id_provider(HexIdProvider::default())
            .build(addr)
            .await
            .map_err(|e| APIError::WebSocketServer(e.to_string()))?;

        rpc.merge(ws_subscription_module(adapter).await).unwrap();

        ret.1 = Some(server.start(rpc))
    }

    Ok(ret)
}
