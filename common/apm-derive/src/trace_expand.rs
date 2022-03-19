use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use fut_ret::PinBoxFutRet;
use proc_macro::TokenStream;
use proc_macro2 as pm2;
use quote::quote;
use syn::{
    parse_macro_input, parse_str, AttributeArgs, Expr, Ident, ItemFn, Lit, Meta, NestedMeta,
    ReturnType, Type,
};

use crate::kv_parser::KeyValue;

const KIND: &str = "kind";
const TRACING_NAME: &str = "name";
const TRACING_TAGS: &str = "tags";
const TRACING_LOGS: &str = "logs";

pub fn expand_trace_span(attr: TokenStream, func: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);
    let func = parse_macro_input!(func as ItemFn);
    let func_vis = &func.vis;
    let func_sig = &func.sig;
    let func_block = &func.block;
    let func_output = &func_sig.output;
    let func_return = PinBoxFutRet::parse(func_output);
    let func_ret_ty = match func_output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };
    let trace_info = TraceAttrs::new(&attr);
    let trace_name = trace_info.trace_name(&func_sig.ident);
    let span_tag_stmts = trace_info.span_tags();
    let span_log_stmts = trace_info.span_logs();

    let func_block_wrapper = if func_return.is_ret_pin_box_fut() && func_return.is_fut_ret_result()
    {
        let ret_ty = func_return.return_type();
        quote! {
            Box::pin(async move {
                let ret: #ret_ty = #func_block.await;

                match span.as_mut() {
                    Some(span) => {
                        match ret.as_ref() {
                            Err(e) => {
                                span.set_tag(|| Tag::new("error", true));
                                span.log(|log| {
                                    log.field(LogField::new(
                                        "error_msg",
                                        e.to_string(),
                                    ));
                                });
                                ret
                            }
                            Ok(_) => {
                                span.set_tag(|| Tag::new("error", false));
                                ret
                            }
                        }
                    }
                    None => ret,
                }
            })
        }
    } else if func_return.is_ret_pin_box_fut() && !func_return.is_fut_ret_result() {
        quote! {
            Box::pin(async move {
                let _ = span;
                #func_block.await
            })
        }
    } else if !func_return.is_ret_pin_box_fut() && is_return_result(func_output) {
        quote! {
            let ret: #func_ret_ty = #func_block;

            match span.as_mut() {
                Some(span) => {
                    match ret.as_ref() {
                        Err(e) => {
                            span.set_tag(|| Tag::new("error", true));
                            span.log(|log| {
                                log.field(LogField::new(
                                    "error_msg",
                                    e.to_string(),
                                ));
                            });
                            ret
                        }
                        Ok(_) => {
                            span.set_tag(|| Tag::new("error", false));
                            ret
                        }
                    }
                }
                None => ret,
            }
        }
    } else {
        quote! { #func_block }
    };

    quote! {
        #[allow(unused_variables, clippy::type_complexity)]
        #func_vis #func_sig {
            use common_apm::tracing::{LogField, SpanContext, Tag, TRACER};

            let mut span_tags: Vec<Tag> = Vec::new();
            #(#span_tag_stmts)*

            let mut span_logs: Vec<LogField> = Vec::new();
            #(#span_log_stmts)*

            let mut span = if let Some(parent_ctx) = ctx.get::<Option<SpanContext>>("parent_span_ctx") {
                if parent_ctx.is_some() {
                    TRACER.load().child_of_span(#trace_name, parent_ctx.clone().unwrap(), span_tags)
                } else {
                    TRACER.load().span(#trace_name, span_tags)
                }
            } else {
                TRACER.load().span(#trace_name, span_tags)
            };

            let ctx = match span.as_mut() {
                Some(span) => {
                    span.log(|log| {
                        for span_log in span_logs.into_iter() {
                            log.field(span_log);
                        }
                    });
                    ctx.with_value("parent_span_ctx", span.context().cloned())
                },
                None => ctx,
            };

            #func_block_wrapper
        }
    }.into()
}

#[derive(Default, Debug)]
pub struct TraceAttrs {
    pub kind:       String,
    pub trace_name: Option<String>,
    pub trace_tags: HashMap<String, String>,
    pub trace_logs: HashMap<String, String>,
}

impl TraceAttrs {
    pub fn new(nested: &[NestedMeta]) -> Self {
        let mut ret = TraceAttrs::default();
        let mut filter = HashSet::new();

        for meta in nested.iter() {
            match meta {
                NestedMeta::Meta(Meta::NameValue(name_value)) => {
                    let ident = name_value.path.segments[0].clone().ident;
                    let val = get_lit_str(&name_value.lit);

                    if filter.contains(&ident) {
                        panic!("Each attribute can only use once");
                    }
                    filter.insert(ident.clone());

                    if ident == KIND {
                        ret.kind = val;
                    } else if ident == TRACING_NAME {
                        ret.trace_name = Some(val);
                    } else if ident == TRACING_TAGS {
                        ret.trace_tags = KeyValue::from_str(val.as_str())
                            .expect("parse tags")
                            .inner();
                    } else if ident == TRACING_LOGS {
                        ret.trace_logs = KeyValue::from_str(val.as_str())
                            .expect("parse logs")
                            .inner();
                    } else {
                        panic!("Invalid attribute");
                    }
                }
                _ => unreachable!("Invalid nested meta"),
            }
        }

        ret
    }

    pub fn span_logs(&self) -> Vec<pm2::TokenStream> {
        self.trace_logs
            .iter()
            .map(|(key, val)| {
                if let Ok(expr) = parse_str::<Expr>(val) {
                    quote! { span_logs.push(LogField::new(#key, (#expr).to_string())); }
                } else {
                    quote! { span_logs.push(LogField::new(#key, #val)); }
                }
            })
            .collect()
    }

    pub fn span_tags(&self) -> Vec<pm2::TokenStream> {
        self.trace_tags
            .iter()
            .map(|(key, val)| {
                if key == KIND {
                    quote! { span_tags.push(Tag::new(#key, #val)); }
                } else if let Ok(expr) = parse_str::<Expr>(val) {
                    quote! { span_tags.push(Tag::new(#key, (#expr).to_string())); }
                } else {
                    quote! { span_tags.push(Tag::new(#key, #val)); }
                }
            })
            .collect()
    }

    pub fn trace_name(&self, func_name: &Ident) -> String {
        if let Some(name) = self.trace_name.clone() {
            self.kind.clone() + "." + name.as_str()
        } else {
            self.kind.clone() + "." + &func_name.to_string()
        }
    }
}

fn get_lit_str(lit: &Lit) -> String {
    match lit {
        Lit::Str(value) => value.value(),
        _ => unreachable!("lit_str"),
    }
}

fn is_return_result(ret_type: &ReturnType) -> bool {
    match ret_type {
        ReturnType::Default => false,

        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .expect("at least one path segment")
                .ident
                .to_string()
                .contains("Result"),
            _ => false,
        },
    }
}
