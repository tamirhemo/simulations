use proc_macro::TokenStream;

mod internal;

#[proc_macro_derive(Internal)]
pub fn internal_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    internal::impl_internal(&ast)
}

#[proc_macro_derive(AgentInternal)]
pub fn sync_internal_queue_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    internal::impl_agent_internal(&ast)
}
