#![allow(non_snake_case)]

use crate::metrics::{
    auto_flush_from, exponential_buckets, make_auto_flush_static_metric, register_counter_vec,
    register_histogram_vec, CounterVec, HistogramVec,
};

use lazy_static::lazy_static;

make_auto_flush_static_metric! {
    pub label_enum RequestKind {
        eth_sendRawTransaction,
        eth_getTransactionByHash,
        eth_getBlockByNumber,
        eth_blockNumber,
        eth_getBlockByHash,
        eth_getTransactionCount,
        eth_getBlockTransactionCountByNumber,
        eth_getBalance,
        eth_call,
        eth_estimateGas,
        eth_chainId,
        net_version,
        eth_getCode,
        eth_getTransactionReceipt,
        net_peerCount,
        net_listening,
        eth_gasPrice,
        eth_syncing,
        eth_getLogs,
        get_block,
        eth_mining,
        eth_feeHistory,
        web3_clientVersion,
        eth_accounts,
        web3_sha3,
        eth_getBlockTransactionCountByHash,
        eth_getTransactionByBlockHashAndIndex,
        eth_getTransactionByBlockNumberAndIndex,
        eth_getStorageAt,
        eth_protocolVersion,
        eth_getUncleByBlockHashAndIndex,
        eth_getUncleByBlockNumberAndIndex,
        eth_getUncleCountByBlockHash,
        eth_getUncleCountByBlockNumber,
        eth_getProof,
    }

    pub label_enum Request_Result {
        success,
        failure,
    }

    pub struct RequestResultCounterVec: LocalCounter {
        "type" => RequestKind,
        "result" => Request_Result,
    }

    pub struct RequestTimeHistogramVec: LocalHistogram {
        "type" => RequestKind,
    }
}

lazy_static! {
    pub static ref API_REQUEST_RESULT_COUNTER_VEC: CounterVec =
        register_counter_vec!(
            "axon_api_request_result_total",
            "Total number of request result",
            &["type", "result"]
        )
        .expect("request result total");
    pub static ref API_REQUEST_TIME_HISTOGRAM_VEC: HistogramVec =
        register_histogram_vec!(
            "axon_api_request_time_cost_seconds",
            "Request process time cost",
            &["type"],
            exponential_buckets(0.001, 2.0, 20).expect("api req time expontial")
        )
        .expect("request time cost");
}

lazy_static! {
    pub static ref API_REQUEST_RESULT_COUNTER_VEC_STATIC: RequestResultCounterVec =
        auto_flush_from!(API_REQUEST_RESULT_COUNTER_VEC, RequestResultCounterVec);
    pub static ref API_REQUEST_TIME_HISTOGRAM_STATIC: RequestTimeHistogramVec =
        auto_flush_from!(API_REQUEST_TIME_HISTOGRAM_VEC, RequestTimeHistogramVec);
}
