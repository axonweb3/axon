use axum::Router;
use std::net::SocketAddr;

pub async fn run_prometheus_server(prometheus_listening_address: SocketAddr) {
    let router = Router::new().route("/metrics", axum::routing::get(get_metrics));
    axum::Server::bind(&prometheus_listening_address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn get_metrics() -> String {
    let metrics_data = match crate::metrics::all_metrics() {
        Ok(data) => data,
        Err(e) => e.to_string().into_bytes(),
    };

    String::from_utf8(metrics_data).unwrap()
}
