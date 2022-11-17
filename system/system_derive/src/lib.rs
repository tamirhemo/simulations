use proc_macro::TokenStream;
use syn;

mod internal;


#[proc_macro_derive(Internal)]
pub fn internal_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    internal::impl_internal(&ast)
}

#[proc_macro_derive(SyncInternalQueue)]
pub fn sync_internal_queue_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    internal::impl_sync_internal_queue(&ast)
}
