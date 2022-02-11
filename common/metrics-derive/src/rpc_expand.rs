use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Ident, ItemFn, Lit, NestedMeta};

pub fn expand_rpc_metrics(attr: TokenStream, func: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);
    let func = parse_macro_input!(func as ItemFn);
    let func_sig = func.sig;
    let func_ident = parse_rpc_method(&attr[0]);
    let func_block = func.block;

    let metrics_field = quote! { #func_ident };

    quote! {
        #func_sig {
            let inst = std::time::Instant::now();
            let ret = #func_block;

            if ret.is_err() {
                common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
                    .#metrics_field
                    .failure
                    .inc();
                return ret;
            }

            common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
                .#metrics_field
                .success
                .inc();

            common_apm::metrics::api::API_REQUEST_TIME_HISTOGRAM_STATIC
                .#metrics_field
                .observe(common_apm::metrics::duration_to_sec(inst.elapsed()));

            ret
        }
    }
    .into()
}

fn parse_rpc_method(input: &NestedMeta) -> proc_macro2::TokenStream {
    let method = match input {
        NestedMeta::Lit(lit) => match lit {
            Lit::Str(api) => Ident::new(&api.value(), Span::call_site()),
            _ => panic!("Invalid lit"),
        },
        _ => panic!("Invalid nested meta"),
    };

    quote! { #method }
}
