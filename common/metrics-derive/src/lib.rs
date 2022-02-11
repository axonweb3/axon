mod rpc_expand;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn metrics_rpc(attr: TokenStream, func: TokenStream) -> TokenStream {
    rpc_expand::expand_rpc_metrics(attr, func)
}
