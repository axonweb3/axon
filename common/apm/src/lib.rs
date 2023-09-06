// https://rust-lang.github.io/rust-clippy/master/index.html#float_cmp
#![allow(clippy::float_cmp)]

pub mod metrics;
pub mod server;
pub mod tracing;

pub use common_apm_derive::{metrics_rpc, trace_span};
pub use minstant::{Anchor, Instant};
pub use prometheus;
pub use prometheus_static_metric;
pub use rustracing_jaeger::span::TraceId;
