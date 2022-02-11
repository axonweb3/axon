mod rpc_expand;

use proc_macro::TokenStream;

/// Expand code for Axon API module to support metrics.
/// ```
/// use std::io::Error;
/// use metrics_derive::metrics_rpc;
/// use protocol::types::{Hash, SignedTransaction};
/// 
/// pub struct RpcExample;
///
/// impl RpcExample {
///     #[metrics_rpc("eth_sendRawTransaction")]
///     fn send_raw_transaction(&self, tx: SignedTransaction) -> Result<Hash, Error> {
///         Ok(tx.transaction.hash)
///     }
/// }
/// ```
/// 
/// The expanded code shown below:
/// ```
/// use std::io::Error;
/// use metrics_derive::metrics_rpc;
/// use protocol::types::{Hash, SignedTransaction};
/// 
/// pub struct ExampleRpc;
/// 
/// impl ExampleRpc {
///     fn send_raw_transaction(&self, tx: SignedTransaction) -> Result<Hash, Error> {
///         let inst = std::time::Instant::now();
///         let ret = { Ok(tx.transaction.hash) };
///         if ret.is_err() {
///             common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
///                 .eth_sendRawTransaction
///                 .failure
///                 .inc();
///             return ret;
///         }
///         common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
///             .eth_sendRawTransaction
///             .success
///             .inc();
///         common_apm::metrics::api::API_REQUEST_TIME_HISTOGRAM_STATIC
///             .eth_sendRawTransaction
///             .observe(common_apm::metrics::duration_to_sec(inst.elapsed()));
///         ret
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn metrics_rpc(attr: TokenStream, func: TokenStream) -> TokenStream {
    rpc_expand::expand_rpc_metrics(attr, func)
}
