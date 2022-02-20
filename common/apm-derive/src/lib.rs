mod rpc_expand;

use proc_macro::TokenStream;

/// Expand code for Axon API module to support metrics.
/// ```ignore
/// use async_trait::async_trait;
/// use jsonrpsee::core::Error;
/// use metrics_derive::metrics_rpc;
///
/// use protocol::types::{Hash, SignedTransaction};
///
/// #[async_trait]
/// pub trait Rpc {
///     async fn send_transaction(&self, tx: SignedTransaction) -> Result<Hash, Error>;
///
///     fn listening(&self) -> Result<bool, Error>;
/// }
///
/// pub struct RpcExample;
///
/// #[async_trait]
/// impl Rpc for RpcExample {
///     #[metrics_rpc("eth_sendRawTransaction")]
///     async fn send_transaction(&self, tx: SignedTransaction) -> Result<Hash, Error> {
///         Ok(tx.transaction.hash)
///     }
///
///     #[metrics_rpc("net_listening")]
///     fn listening(&self) -> Result<bool, Error> {
///         Ok(false)
///     }
/// }
/// ```
///
/// The expanded code shown below:
/// ```ignore
/// use async_trait::async_trait;
/// use jsonrpsee::core::Error;
/// use metrics_derive::metrics_rpc;
/// use protocol::types::{Hash, SignedTransaction};
/// pub trait Rpc {
///     #[must_use]
///     #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
///     fn send_transaction<'life0, 'async_trait>(
///         &'life0 self,
///         tx: SignedTransaction,
///     ) -> ::core::pin::Pin<
///         Box<
///             dyn ::core::future::Future<Output = Result<Hash, Error>>
///                 + ::core::marker::Send
///                 + 'async_trait,
///         >,
///     >
///     where
///         'life0: 'async_trait,
///         Self: 'async_trait;
///
///     fn listening(&self) -> Result<bool, Error>;
/// }
///
/// pub struct RpcExample;
///
/// impl Rpc for RpcExample {
///     fn send_transaction<'life0, 'async_trait>(
///         &'life0 self,
///         tx: SignedTransaction,
///     ) -> ::core::pin::Pin<
///         Box<
///             dyn ::core::future::Future<Output = Result<Hash, Error>>
///                 + ::core::marker::Send
///                 + 'async_trait,
///         >,
///     >
///     where
///         'life0: 'async_trait,
///         Self: 'async_trait,
///     {
///         Box::pin(async move {
///             let inst = std::time::Instant::now();
///             let ret: Result<Hash, Error> = {
///                 Box::pin(async move {
///                     if let ::core::option::Option::Some(__ret) =
///                         ::core::option::Option::None::<Result<Hash, Error>>
///                     {
///                         return __ret;
///                     }
///                     let __self = self;
///                     let tx = tx;
///                     let __ret: Result<Hash, Error> = { Ok(tx.transaction.hash) };
///                     #[allow(unreachable_code)]
///                     __ret
///                 })
///             }
///             .await;
///
///             if ret.is_err() {
///                 common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
///                     .eth_sendRawTransaction
///                     .failure
///                     .inc();
///                 return ret;
///             }
///
///             common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
///                 .eth_sendRawTransaction
///                 .success
///                 .inc();
///             common_apm::metrics::api::API_REQUEST_TIME_HISTOGRAM_STATIC
///                 .eth_sendRawTransaction
///                 .observe(common_apm::metrics::duration_to_sec(common_apm::elapsed(inst)));
///             ret
///         })
///     }
///
///     fn listening(&self) -> Result<bool, Error> {
///         let inst = std::time::Instant::now();
///         let ret: Result<bool, Error> = { Ok(false) };
///
///         if ret.is_err() {
///             common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
///                 .net_listening
///                 .failure
///                 .inc();
///             return ret;
///         }
///
///         common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
///             .net_listening
///             .success
///             .inc();
///         common_apm::metrics::api::API_REQUEST_TIME_HISTOGRAM_STATIC
///             .net_listening
///             .observe(common_apm::metrics::duration_to_sec(common_apm::elapsed(inst)));
///         ret
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn metrics_rpc(attr: TokenStream, func: TokenStream) -> TokenStream {
    rpc_expand::expand_rpc_metrics(attr, func)
}
