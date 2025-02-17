use std::sync::atomic::{AtomicU64, Ordering};
use quote::quote;

use proc_macro::TokenStream;
use syn::parse_macro_input;

static RESOURCE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn impl_resource_id_macro(ast: &syn::DeriveInput) -> TokenStream {
    let id = RESOURCE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    let name = &ast.ident;

    let gen = quote! {
        impl #name {
            pub const RESOURCE_ID: u64 = #id;
        }

        impl Resource for #name {
            fn get_resource_id(&self) -> u64 {
                #id
            }
        }
    };
    gen.into()
}

pub fn impl_get_resource_id_macro(input: TokenStream) -> TokenStream {
    let type_result = parse_macro_input!(input as syn::Type);

    let result = quote! {
        <#type_result>::RESOURCE_ID
    };

    TokenStream::from(result)
}