use proc_macro::TokenStream;
use syn;

mod agent;

use agent::impl_internal;

#[proc_macro_derive(Internal)]
pub fn internal_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_internal(&ast)
}

#[proc_macro_derive(SyncInternalQueue)]
pub fn sync_internal_queue_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_internal(&ast)
}
