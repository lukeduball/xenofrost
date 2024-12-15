use std::sync::atomic::{AtomicU64, Ordering};
use quote::quote;

use proc_macro::TokenStream;

static COMPONENT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn impl_component_id_macro(ast: &syn::DeriveInput) -> TokenStream {
    let id = COMPONENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    let name = &ast.ident;

    let gen = quote! {
        impl Component for #name {
            const ID: u64 = #id;

            fn get_component_id(&self) -> u64 {
                Self::ID
            }

            fn component_id() -> u64 {
                Self::ID
            }
        }
    };
    gen.into()
}