#![allow(clippy::uninlined_format_args)]

mod kv_parser;
mod rpc_expand;
mod trace_expand;

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

/// Expand code for Axon API module to support metrics.
/// ```ignore
/// use async_trait::async_trait;
/// use common_apm_derive::trace_span;
///
/// use protocol::{traits::Context, types::SignedTransaction, ProtocolResult};
///
/// #[async_trait]
/// pub trait ToTrace {
///     async fn store(&self, ctx: Context, txs: Vec<SignedTransaction>) -> ProtocolResult<()>;
///
///     fn version(&self, ctx: Context) -> String;
/// }
///
/// pub struct TraceExample;
///
/// #[async_trait]
/// impl ToTrace for TraceExample {
///     #[trace_span(kind = "trace", logs = "{tx_len: txs.len()}")]
///     async fn store(&self, ctx: Context, txs: Vec<SignedTransaction>) -> ProtocolResult<()> {
///         debug_assert!(txs.len() == 1);
///         Ok(())
///     }
///
///     #[trace_span(kind = "trace")]
///     fn version(&self, ctx: Context) -> String {
///         "0.1.0".to_string()
///     }
/// }
/// ```
/// The expanded code shown below:
/// ```ignore
/// use async_trait::async_trait;
/// use common_apm_derive::trace_span;
/// use protocol::{traits::Context, types::SignedTransaction, ProtocolResult};
///
/// pub trait ToTrace {
///     #[must_use]
///     #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
///     fn store<'life0, 'async_trait>(
///         &'life0 self,
///         ctx: Context,
///         txs: Vec<SignedTransaction>,
///     ) -> ::core::pin::Pin<
///         Box<
///             dyn ::core::future::Future<Output = ProtocolResult<()>>
///                 + ::core::marker::Send
///                 + 'async_trait,
///         >,
///     >
///     where
///         'life0: 'async_trait,
///         Self: 'async_trait;
///     fn version(&self, ctx: Context) -> String;
/// }
///
/// pub struct TraceExample;
///
/// impl ToTrace for TraceExample {
///     fn store<'life0, 'async_trait>(
///         &'life0 self,
///         ctx: Context,
///         txs: Vec<SignedTransaction>,
///     ) -> ::core::pin::Pin<
///         Box<
///             dyn ::core::future::Future<Output = ProtocolResult<()>>
///                 + ::core::marker::Send
///                 + 'async_trait,
///         >,
///     >
///     where
///         'life0: 'async_trait,
///         Self: 'async_trait,
///     {
///         use common_apm::tracing::{LogField, SpanContext, Tag, TRACER};
///         let mut span_tags: Vec<Tag> = Vec::new();
///         let mut span_logs: Vec<LogField> = Vec::new();
///         span_logs.push(LogField::new("tx_len", (txs.len()).to_string()));
///         let mut span = if let Some(parent_ctx) = ctx.get::<Option<SpanContext>>("parent_span_ctx") {
///             if parent_ctx.is_some() {
///                 TRACER
///                     .load()
///                     .child_of_span("trace.store", parent_ctx.clone().unwrap(), span_tags)
///             } else {
///                 TRACER.load().span("trace.store", span_tags)
///             }
///         } else {
///             TRACER.load().span("trace.store", span_tags)
///         };
///         let ctx = match span.as_mut() {
///             Some(span) => {
///                 span.log(|log| {
///                     for span_log in span_logs.into_iter() {
///                         log.field(span_log);
///                     }
///                 });
///                 ctx.with_value("parent_span_ctx", span.context().cloned())
///             }
///             None => ctx,
///         };
///         Box::pin(async move {
///             let ret: ProtocolResult<()> = {
///                 Box::pin(async move {
///                     if let ::core::option::Option::Some(__ret) =
///                         ::core::option::Option::None::<ProtocolResult<()>>
///                     {
///                         return __ret;
///                     }
///                     let __self = self;
///                     let ctx = ctx;
///                     let txs = txs;
///                     let __ret: ProtocolResult<()> = {
///                         if true {
///                             if !(txs.len() == 1) {
///                                 ::core::panicking::panic("assertion failed: txs.len() == 1")
///                             };
///                         };
///                         Ok(())
///                     };
///                     #[allow(unreachable_code)]
///                     __ret
///                 })
///             }
///             .await;
///             match span.as_mut() {
///                 Some(span) => match ret.as_ref() {
///                     Err(e) => {
///                         span.set_tag(|| Tag::new("error", true));
///                         span.log(|log| {
///                             log.field(LogField::new("error_msg", e.to_string()));
///                         });
///                         ret
///                     }
///                     Ok(_) => {
///                         span.set_tag(|| Tag::new("error", false));
///                         ret
///                     }
///                 },
///                 None => ret,
///             }
///         })
///     }
///
///     fn version(&self, ctx: Context) -> String {
///         use common_apm::tracing::{LogField, SpanContext, Tag, TRACER};
///         let mut span_tags: Vec<Tag> = Vec::new();
///         let mut span_logs: Vec<LogField> = Vec::new();
///         let mut span = if let Some(parent_ctx) = ctx.get::<Option<SpanContext>>("parent_span_ctx") {
///             if parent_ctx.is_some() {
///                 TRACER
///                     .load()
///                     .child_of_span("trace.version", parent_ctx.clone().unwrap(), span_tags)
///             } else {
///                 TRACER.load().span("trace.version", span_tags)
///             }
///         } else {
///             TRACER.load().span("trace.version", span_tags)
///         };
///         let ctx = match span.as_mut() {
///             Some(span) => {
///                 span.log(|log| {
///                     for span_log in span_logs.into_iter() {
///                         log.field(span_log);
///                     }
///                 });
///                 ctx.with_value("parent_span_ctx", span.context().cloned())
///             }
///             None => ctx,
///         };
///         {
///             "0.1.0".to_string()
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn trace_span(attr: TokenStream, func: TokenStream) -> TokenStream {
    trace_expand::expand_trace_span(attr, func)
}
