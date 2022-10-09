use fut_ret::PinBoxFutRet;
use proc_macro::TokenStream;
use proc_macro2 as pm2;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Ident, ItemFn, Lit, NestedMeta, ReturnType};

pub fn expand_rpc_metrics(attr: TokenStream, func: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);
    let func = parse_macro_input!(func as ItemFn);
    let func_sig = &func.sig;
    let func_ident = parse_rpc_method(&attr[0]);
    let func_block = &func.block;
    let func_output = &func_sig.output;
    let func_return = PinBoxFutRet::parse(func_output);
    let func_ret_ty = match func_output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };
    let ret_ty = func_return.return_type();
    let args = func_sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(inner) => match &*inner.pat {
                syn::Pat::Ident(pat) => Some(pat.ident.clone()),
                _ => None,
            },
        })
        .collect::<Vec<_>>();

    let fn_name = func_ident.to_string();
    let mut debug = if args.is_empty() {
        "call rpc {}".to_string()
    } else {
        "call rpc {} with args,".to_string()
    };
    for (index, name) in args.iter().enumerate() {
        if index != 0 {
            debug.push(',')
        }
        debug.push_str(&format!(" {}: {{:?}}", name.to_string()));
    }

    let func_block_wrapper = if func_return.is_ret_pin_box_fut() {
        quote! {
            Box::pin(async move {
                let inst = common_apm::Instant::now();

                log::debug!(#debug, #fn_name, #( #args, )*);

                let ret: #ret_ty = #func_block.await;

                if ret.is_err() {
                    common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
                        .#func_ident
                        .failure
                        .inc();
                    return ret;
                }

                common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
                    .#func_ident
                    .success
                    .inc();
                common_apm::metrics::api::API_REQUEST_TIME_HISTOGRAM_STATIC
                    .#func_ident
                    .observe(common_apm::metrics::duration_to_sec(inst.elapsed()));

                ret
            })
        }
    } else {
        quote! {
            let inst = common_apm::Instant::now();

            log::debug!(#debug, #fn_name, #( #args, )*);

            let ret: #func_ret_ty = #func_block;

            if ret.is_err() {
                common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
                    .#func_ident
                    .failure
                    .inc();
                return ret;
            }

            common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
                .#func_ident
                .success
                .inc();
            common_apm::metrics::api::API_REQUEST_TIME_HISTOGRAM_STATIC
                .#func_ident
                .observe(common_apm::metrics::duration_to_sec(inst.elapsed()));

            ret
        }
    };

    quote! {
        #func_sig {
            #func_block_wrapper
        }
    }
    .into()
}

fn parse_rpc_method(input: &NestedMeta) -> pm2::TokenStream {
    let method = match input {
        NestedMeta::Lit(Lit::Str(api)) => Ident::new(&api.value(), pm2::Span::call_site()),
        _ => unreachable!("Invalid nested meta"),
    };

    quote! { #method }
}
