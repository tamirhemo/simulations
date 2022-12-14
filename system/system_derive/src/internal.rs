use proc_macro::TokenStream;
use quote::quote;

/// A function fo automaticaly producing match arms for a method
fn match_arms(
    name: &syn::Ident,
    data_enum: &syn::DataEnum,
    method_token: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let mut match_arms = proc_macro2::TokenStream::new();

    match_arms.extend(
        data_enum
            .variants
            .iter()
            .map(|variant| &variant.ident)
            .map(|variant| {
                quote!(
                    #name::#variant(internal) => internal.#method_token,
                )
            }),
    );

    match_arms
}

/// Deriving ActorInternal for an enum of types implementing Internal
///
/// This macto assumes Message, Key, Channel are joint for all the enum variants
pub fn impl_actor_internal(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let generics = &ast.generics.params;
    let where_clause = &ast.generics.where_clause;

    let data_enum = match &ast.data {
        syn::Data::Enum(data_enum) => data_enum,
        _ => panic!("Can't match structure"),
    };

    let first_field = match &data_enum.variants.first().unwrap().fields {
        syn::Fields::Unnamed(field) => field.unnamed.first().unwrap(),
        _ => unreachable!(),
    };

    // Tokens for methods
    let outgoint_key_token = quote!(new_outgoing_key(key));
    let incoming_key_token = quote!(new_incoming_key(key));
    let start_token = quote!(start(tx));
    let process_message_token = quote!(process_message(message, tx));

    let make_arms = |token| match_arms(name, data_enum, token);

    // Match arms for methods
    let outgoint_key_arms = make_arms(outgoint_key_token);
    let incoming_key_arms = make_arms(incoming_key_token);
    let start_arms = make_arms(start_token);
    let process_message_arms = make_arms(process_message_token);

    let mut trypl = proc_macro2::TokenStream::new();
    trypl.extend(vec![quote!(let x :u32 = 5; )]);

    let gen = quote! {
        impl<#generics> ActorInternal for #name<#generics>
        #where_clause {
            type Message = <#first_field as ActorInternal>::Message;
            type Key = <#first_field as ActorInternal>::Key;
            type Error = <#first_field as ActorInternal>::Error;

            fn new_outgoing_key(&mut self, key: &Self::Key) {
                #trypl
                match self {
                    #outgoint_key_arms
                }
            }

            fn new_incoming_key(&mut self, key: &Self::Key) {
                match self {
                    #incoming_key_arms
                }
            }

            fn start<SenderGenericName: Sender<Key = Self::Key, Message = Self::Message>>(
                &mut self,
                tx: &mut SenderGenericName,
            ) -> Result<NextState<Self::Message>, Self::Error> {
                match self {
                    #start_arms
                }
            }

            fn process_message<SenderGenericName: Sender<Key = Self::Key, Message = Self::Message>>(
                &mut self,
                message: Option<Self::Message>,
                tx: &mut SenderGenericName,
            ) -> Result<NextState<Self::Message>, Self::Error> {
                match self {
                    #process_message_arms
                }
            }
        }
    };

    gen.into()
}

/// Deriving Internal for an enum of types implementing Internal
///
/// This macto assumes Message, Key, Channel are joint for all the enum variants
pub fn impl_internal(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let generics = &ast.generics.params;
    let where_clause = &ast.generics.where_clause;

    let data_enum = match &ast.data {
        syn::Data::Enum(data_enum) => data_enum,
        _ => panic!("Can't match structure"),
    };

    let first_field = match &data_enum.variants.first().unwrap().fields {
        syn::Fields::Unnamed(field) => field.unnamed.first().unwrap(),
        _ => unreachable!(),
    };

    // Tokens for methods
    let outgoint_key_token = quote!(new_outgoing_key(key));
    let incoming_key_token = quote!(new_incoming_key(key));
    let start_token = quote!(start());
    let process_message_token = quote!(process_message(message));

    let make_arms = |token| match_arms(name, data_enum, token);

    // Match arms for methods
    let outgoint_key_arms = make_arms(outgoint_key_token);
    let incoming_key_arms = make_arms(incoming_key_token);
    let start_arms = make_arms(start_token);
    let process_message_arms = make_arms(process_message_token);

    let mut trypl = proc_macro2::TokenStream::new();
    trypl.extend(vec![quote!(let x :u32 = 5; )]);

    let gen = quote! {
        impl<#generics> Internal for #name<#generics>
        #where_clause {
            type Message = <#first_field as Internal>::Message;
            type Key = <#first_field as Internal>::Key;
            type Queue = <#first_field as Internal>::Queue;
            type Error = <#first_field as Internal>::Error;

            fn new_outgoing_key(&mut self, key: &Self::Key) {
                #trypl
                match self {
                    #outgoint_key_arms
                }
            }

            fn new_incoming_key(&mut self, key: &Self::Key) {
                match self {
                    #incoming_key_arms
                }
            }

            fn start(&mut self) -> Self::Queue {
                match self {
                    #start_arms
                }
            }

            fn process_message( &mut self, message: Option<Self::Message>) -> Self::Queue {
                match self {
                    #process_message_arms
                }
            }
        }
    };

    gen.into()
}
