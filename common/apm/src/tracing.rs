pub use rustracing::{log::LogField, tag::Tag};
pub use rustracing_jaeger::span::SpanContext;

use std::net::SocketAddr;
use std::sync::Arc;

use arc_swap::ArcSwap;
use beef::lean::Cow;
use rustracing::sampler::AllSampler;
use rustracing::ErrorKind;
use rustracing_jaeger::reporter::JaegerCompactReporter;
use rustracing_jaeger::span::{
    Span, SpanContextState, SpanContextStateBuilder, SpanSender, TraceId,
};
use rustracing_jaeger::AsyncTracer;
use trackable::error::{ErrorKindExt, TrackableError};
use trackable::{track, track_panic};

use protocol::tokio::{self, sync::mpsc::channel};
use protocol::traits::Context;

const SPAN_CHANNEL_SIZE: usize = 1024 * 1024;
const DEFAULT_SPAN_BATCH_SIZE: usize = 20;

lazy_static::lazy_static! {
    pub static ref TRACER: ArcSwap<AxonTracer> = ArcSwap::new(Arc::new(AxonTracer::default()));
}

pub struct AxonTracer(AsyncTracer);

impl Default for AxonTracer {
    fn default() -> Self {
        let (tracer, _handle) = AsyncTracer::new(AllSampler);
        AxonTracer(tracer)
    }
}

impl AxonTracer {
    pub fn with_sender(sender: SpanSender) -> Self {
        AxonTracer(AsyncTracer::with_sender(AllSampler, sender))
    }

    pub fn child_of_span<N: Into<Cow<'static, str>>>(
        &self,
        opt_name: N,
        parent_ctx: SpanContext,
        tags: Vec<Tag>,
    ) -> Option<Span> {
        let mut span = self.0.span(opt_name);
        for tag in tags.into_iter() {
            span = span.tag(tag);
        }
        Some(span.child_of(&parent_ctx).start())
    }

    pub fn span<N: Into<Cow<'static, str>>>(&self, opt_name: N, tags: Vec<Tag>) -> Option<Span> {
        let mut span = self.0.span(opt_name);
        for tag in tags.into_iter() {
            span = span.tag(tag);
        }
        Some(span.start())
    }

    pub fn new_state(trace_id: TraceId, span_id: u64) -> SpanContextState {
        SpanContextStateBuilder::new()
            .trace_id(trace_id)
            .span_id(span_id)
            .finish()
    }

    pub fn span_state(ctx: &Context) -> Option<SpanContextState> {
        if let Some(Some(parent_ctx)) = ctx.get::<Option<SpanContext>>("parent_span_ctx") {
            Some(parent_ctx.state().to_owned())
        } else {
            None
        }
    }

    pub fn inject_span_state(ctx: Context, span_state: SpanContextState) -> Context {
        let span = SpanContext::new(span_state, vec![]);
        ctx.with_value::<Option<SpanContext>>("parent_span_ctx", Some(span))
    }

    pub fn str_to_trace(s: &str) -> Result<TraceId, TrackableError<ErrorKind>> {
        if s.len() <= 16 {
            let low =
                track!(u64::from_str_radix(s, 16).map_err(AxonTracer::from_parse_int_error,))?;
            Ok(TraceId { high: 0, low })
        } else if s.len() <= 32 {
            let (high, low) = s.as_bytes().split_at(s.len() - 16);
            let high = track!(std::str::from_utf8(high).map_err(AxonTracer::from_utf8_error))?;
            let high =
                track!(u64::from_str_radix(high, 16).map_err(AxonTracer::from_parse_int_error,))?;

            let low = track!(std::str::from_utf8(low).map_err(AxonTracer::from_utf8_error))?;
            let low =
                track!(u64::from_str_radix(low, 16).map_err(AxonTracer::from_parse_int_error,))?;
            Ok(TraceId { high, low })
        } else {
            track_panic!(ErrorKind::InvalidInput, "s={:?}", s)
        }
    }

    fn from_parse_int_error(f: std::num::ParseIntError) -> TrackableError<rustracing::ErrorKind> {
        TrackableError::new(ErrorKind::InvalidInput, f)
    }

    fn from_utf8_error(f: std::str::Utf8Error) -> TrackableError<rustracing::ErrorKind> {
        ErrorKind::InvalidInput.cause(f)
    }
}

pub fn global_tracer_register(service_name: &str, udp_addr: SocketAddr, batch_size: Option<usize>) {
    let (span_tx, mut span_rx) = channel(SPAN_CHANNEL_SIZE);
    TRACER.swap(Arc::new(AxonTracer::with_sender(span_tx)));

    let reporter = new_jaeger_reporter(service_name, udp_addr);
    let batch_size = batch_size.unwrap_or(DEFAULT_SPAN_BATCH_SIZE);

    tokio::spawn(async move {
        let mut batch_spans = Vec::with_capacity(batch_size);

        loop {
            if let Some(finished_span) = span_rx.recv().await {
                batch_spans.push(finished_span);

                if batch_spans.len() >= batch_size {
                    let enough_spans = batch_spans.drain(..).collect::<Vec<_>>();
                    if let Err(err) = reporter.report(&enough_spans) {
                        log::warn!("jaeger report {:?}", err);
                    }
                }
            }
        }
    });
}

fn new_jaeger_reporter(service_name: &str, udp_addr: SocketAddr) -> JaegerCompactReporter {
    let mut reporter = JaegerCompactReporter::new(service_name).unwrap();
    reporter.set_agent_addr(udp_addr);
    reporter
}
