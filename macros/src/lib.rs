use component_macros::impl_component_id_macro;
use proc_macro::TokenStream;
use query_macros::impl_world_query_macro;

extern crate proc_macro;

mod macro_utilities;
mod component_macros;
mod query_macros;

#[proc_macro_derive(Component)]
pub fn derive_component_id_macro(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_component_id_macro(&ast)
}

#[proc_macro]
pub fn world_query(input: TokenStream) -> TokenStream {
    impl_world_query_macro(input)
}