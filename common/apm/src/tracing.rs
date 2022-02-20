use std::net::SocketAddr;
use std::sync::Arc;

use arc_swap::ArcSwap;
use beef::lean::Cow;
use rustracing::{sampler::AllSampler, tag::Tag};
use rustracing_jaeger::reporter::JaegerCompactReporter;
use rustracing_jaeger::span::{
    Span, SpanContext, SpanContextState, SpanContextStateBuilder, SpanSender, TraceId,
};
use rustracing_jaeger::AsyncTracer;

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
}

pub fn global_tracer_register(service_name: &str, udp_addr: SocketAddr, batch_size: Option<usize>) {
    let (span_tx, mut span_rx) = channel(SPAN_CHANNEL_SIZE);
    let batch_size = batch_size.unwrap_or(DEFAULT_SPAN_BATCH_SIZE);
    let mut reporter = JaegerCompactReporter::new(service_name).unwrap();
    reporter.set_agent_addr(udp_addr);
    TRACER.swap(Arc::new(AxonTracer::with_sender(span_tx)));

    tokio::spawn(async move {
        let mut batch_spans = Vec::with_capacity(batch_size);

        loop {
            if let Some(finished_span) = span_rx.recv().await {
                batch_spans.push(finished_span);

                if batch_spans.len() >= batch_size {
                    let enough_spans = batch_spans.drain(..).collect::<Vec<_>>();
                    if let Err(err) = reporter.report(&enough_spans) {
                        log::warn!("jaeger report {}", err);
                    }
                }
            }
        }
    });
}
