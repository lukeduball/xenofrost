use std::sync::atomic::{AtomicU64, Ordering};
use quote::quote;

use proc_macro::TokenStream;
use syn::parse_macro_input;

static COMPONENT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn impl_component_id_macro(ast: &syn::DeriveInput) -> TokenStream {
    let id = COMPONENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    let name = &ast.ident;

    let gen = quote! {
        impl #name {
            pub const COMPONENT_ID: u64 = #id;
        }

        impl Component for #name {
            fn get_component_id(&self) -> u64 {
                #id
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
    gen.into()
}

pub fn impl_get_component_id_macro(input: TokenStream) -> TokenStream {
    let type_result = parse_macro_input!(input as syn::Type);

    let result = quote! {
        <#type_result>::COMPONENT_ID
    };

    TokenStream::from(result)
}