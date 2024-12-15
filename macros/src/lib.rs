use component_macros::impl_component_id_macro;
use proc_macro::TokenStream;

extern crate proc_macro;

mod component_macros;

#[proc_macro_derive(Component)]
pub fn derive_component_id_macro(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_component_id_macro(&ast)
}