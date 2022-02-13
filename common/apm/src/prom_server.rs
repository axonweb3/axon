use axum::Router;

pub fn prom_server() -> Router {
    Router::new().route("/metrics", axum::routing::get(get_metrics))
}

async fn get_metrics() -> String {
    let metrics_data = match super::metrics::all_metrics() {
        Ok(data) => data,
        Err(e) => e.to_string().into_bytes(),
    };

    String::from_utf8(metrics_data).unwrap()
}
