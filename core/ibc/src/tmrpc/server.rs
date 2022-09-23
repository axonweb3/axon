use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tendermint::abci;
use tendermint::evidence::Evidence;
use tendermint_rpc::query::Query;
use tendermint_rpc::{Order, PageNumber, Paging, PerPage};
use warp::reject::Rejection;
use warp::{http::Response, Filter};

use protocol::traits::TendermintRpc;
use protocol::ProtocolResult;

pub async fn run_tm_rpc<T>(adapter: T, laddr: String)
where
    T: TendermintRpc + 'static,
{
    let adapter = Arc::new(adapter);
    let service = Arc::clone(&adapter);
    let health = warp::path("health").and_then(move || {
        let t = Arc::clone(&service);
        async move {
            let response = Wrapper::from_result(t.health().await).to_response();
            let res: Result<_, Rejection> = Ok(response);
            res
        }
    });
    let service = Arc::clone(&adapter);
    let status = warp::path("status").and_then(move || {
        let t = Arc::clone(&service);
        async move {
            let response = Wrapper::from_result(t.status().await).to_response();
            let res: Result<_, Rejection> = Ok(response);
            res
        }
    });
    let service = Arc::clone(&adapter);
    let net_info = warp::path("net_info").and_then(move || {
        let t = Arc::clone(&service);
        async move {
            let response = Wrapper::from_result(t.net_info().await).to_response();
            let res: Result<_, Rejection> = Ok(response);
            res
        }
    });
    let service = Arc::clone(&adapter);
    let blockchain = warp::get()
        .and(warp::path("blockchain"))
        .and(warp::query::<HashMap<String, u32>>())
        .and_then(move |p: HashMap<String, u32>| {
            let t = Arc::clone(&service);
            async move {
                let min = p.get("min");
                let max = p.get("max");
                let res: Result<_, Rejection> = Ok(match (min, max) {
                    (Some(min), Some(max)) => Wrapper::from_result(t.blockchain(*min, *max).await),
                    (None, _) => Wrapper::from_err_msg("param min is required".to_string()),
                    (_, None) => Wrapper::from_err_msg("param max is required".to_string()),
                }
                .to_response());
                res
            }
        });
    let service = Arc::clone(&adapter);
    let block = warp::get()
        .and(warp::path("block"))
        .and(warp::query::<HashMap<String, u32>>())
        .and_then(move |p: HashMap<String, u32>| {
            let t = Arc::clone(&service);
            async move {
                let res: Result<_, Rejection> = Ok(match p.get("height") {
                    Some(h) => Wrapper::from_result(t.block(Some(*h)).await),
                    None => Wrapper::from_result(t.block::<u32>(None).await),
                }
                .to_response());
                res
            }
        });
    let service = Arc::clone(&adapter);
    let block_results = warp::get()
        .and(warp::path("block_results"))
        .and(warp::query::<HashMap<String, u32>>())
        .and_then(move |p: HashMap<String, u32>| {
            let t = Arc::clone(&service);
            async move {
                let res: Result<_, Rejection> = Ok(match p.get("height") {
                    Some(h) => Wrapper::from_result(t.block_results(Some(*h)).await),
                    None => Wrapper::from_result(t.block_results::<u32>(None).await),
                }
                .to_response());
                res
            }
        });
    let service = Arc::clone(&adapter);
    let commit = warp::get()
        .and(warp::path("commit"))
        .and(warp::query::<HashMap<String, u32>>())
        .and_then(move |p: HashMap<String, u32>| {
            let t = Arc::clone(&service);
            async move {
                let res: Result<_, Rejection> = Ok(match p.get("height") {
                    Some(h) => Wrapper::from_result(t.commit(Some(*h)).await),
                    None => Wrapper::from_result(t.commit::<u32>(None).await),
                }
                .to_response());
                res
            }
        });
    let service = Arc::clone(&adapter);
    let validators = warp::get()
        .and(warp::path("validators"))
        .and(warp::query::<ValidatorsReqParams>())
        .and_then(move |p: ValidatorsReqParams| {
            let t = Arc::clone(&service);
            async move {
                let page_number = PageNumber::from(p.page_number.unwrap_or(1));
                let per_page = PerPage::from(p.per_page.unwrap_or(30));
                let result = t
                    .validators(p.height, Paging::Specific {
                        page_number,
                        per_page,
                    })
                    .await;
                let res: Result<_, Rejection> = Ok(Wrapper::from_result(result).to_response());
                res
            }
        });
    // Figure out: AppState
    // let genesis = warp::path("genesis").map(||
    // Wrapper::from_result(t.genesis()));
    // let t = Arc::clone(&adapter);
    let service = Arc::clone(&adapter);
    let consensus_state = warp::path("consensus_state").and_then(move || {
        let t = Arc::clone(&service);
        async move {
            let res: Result<_, Rejection> =
                Ok(Wrapper::from_result(t.consensus_state().await).to_response());
            res
        }
    });
    let service = Arc::clone(&adapter);
    let consensus_params = warp::get()
        .and(warp::path("consensus_params"))
        .and(warp::query::<HashMap<String, u32>>())
        .and_then(move |p: HashMap<String, u32>| {
            let t = Arc::clone(&service);
            async move {
                let res: Result<_, Rejection> = Ok(match p.get("height") {
                    Some(h) => Wrapper::from_result(t.consensus_params(Some(*h)).await),
                    None => Wrapper::from_result(t.consensus_params::<u32>(None).await),
                }
                .to_response());
                res
            }
        });
    let service = Arc::clone(&adapter);
    let tx = warp::get()
        .and(warp::path("tx"))
        .and(warp::query::<TxReqParams>())
        .and_then(move |p: TxReqParams| {
            let t = Arc::clone(&service);
            async move {
                let prove = p.prove.unwrap_or(false);
                let hash = p.hash;
                let res: Result<_, Rejection> =
                    Ok(Wrapper::from_result(t.tx(hash, prove).await).to_response());
                res
            }
        });
    let service = Arc::clone(&adapter);
    let tx_search = warp::get()
        .and(warp::path("tx_search"))
        .and(warp::query::<TxSearchReqParams>())
        .and_then(move |p: TxSearchReqParams| {
            let t = Arc::clone(&service);
            async move {
                let query = p.query.parse::<Query>();
                let res: Result<_, Rejection> = Ok(match query {
                    Ok(q) => {
                        let prove = p.prove.unwrap_or(false);
                        let page = p.page.unwrap_or(1);
                        let per_page = p.per_page.unwrap_or(30);
                        let order = p.order_by.unwrap_or(Order::Ascending);
                        Wrapper::from_result(t.tx_search(q, prove, page, per_page, order).await)
                    }
                    Err(e) => Wrapper::from_err_msg(e.to_string()),
                }
                .to_response());
                res
            }
        });
    let service = Arc::clone(&adapter);
    let broadcast_evidence = warp::get()
        .and(warp::path("broadcast_evidence"))
        .and(warp::query::<BroadcastEvidenceReqParams>())
        .and_then(move |p: BroadcastEvidenceReqParams| {
            let t = Arc::clone(&service);
            async move {
                let res: Result<_, Rejection> =
                    Ok(Wrapper::from_result(t.broadcast_evidence(p.evidence).await).to_response());
                res
            }
        });
    let router = health
        .or(status)
        .or(net_info)
        .or(blockchain)
        .or(block)
        .or(block_results)
        .or(commit)
        .or(consensus_state)
        .or(consensus_params)
        .or(tx)
        .or(tx_search)
        .or(validators)
        .or(broadcast_evidence);
    warp::serve(router)
        .run(laddr.parse::<SocketAddr>().unwrap())
        .await
}

#[derive(Debug, Deserialize, Serialize)]
struct BroadcastEvidenceReqParams {
    pub evidence: Evidence,
}

#[derive(Debug, Deserialize, Serialize)]
struct ValidatorsReqParams {
    pub height:      u32,
    pub page_number: Option<usize>,
    pub per_page:    Option<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TxReqParams {
    hash:  abci::transaction::Hash,
    prove: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct TxSearchReqParams {
    pub query:    String,
    pub prove:    Option<bool>,
    pub page:     Option<u32>,
    pub per_page: Option<u8>,
    pub order_by: Option<Order>,
}

// Bascially the same as terdemint_rpc::response::Wrapper. Just public the
// fields to build a wrapper.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Wrapper<R: Serialize> {
    pub jsonrpc: String,
    pub id:      i32,
    pub result:  Option<R>,
    pub error:   Option<String>,
}

impl<R: Serialize> Wrapper<R> {
    pub fn from_result(result: ProtocolResult<R>) -> Self {
        match result {
            Ok(res) => Wrapper {
                jsonrpc: "2.0".to_string(),
                id:      0,
                result:  Some(res),
                error:   None,
            },
            Err(e) => Wrapper {
                jsonrpc: "2.0".to_string(),
                id:      0,
                result:  None,
                error:   Some(e.to_string()),
            },
        }
    }

    pub fn from_err_msg(msg: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id:      0,
            result:  None,
            error:   Some(msg),
        }
    }

    pub fn to_response(self) -> Response<String> {
        let body = serde_json::to_string(&self).unwrap();
        Response::new(body)
    }
}
