use component_macros::{impl_component_id_macro, impl_get_component_id_macro, impl_get_number_of_components};
use proc_macro::TokenStream;
use query_macros::impl_world_query_macro;
use resource_macros::{impl_get_resource_id_macro, impl_query_resource, impl_resource_id_macro, impl_get_number_of_resources};

extern crate proc_macro;

mod macro_utilities;
mod component_macros;
mod query_macros;
mod resource_macros;

#[proc_macro_derive(Component)]
pub fn derive_component_id_macro(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_component_id_macro(&ast)
}

#[proc_macro]
pub fn get_component_id(input: TokenStream) -> TokenStream {
    impl_get_component_id_macro(input)
}

#[proc_macro]
pub fn get_number_of_components(input: TokenStream) -> TokenStream {
    impl_get_number_of_components(input)
}
 
#[proc_macro_derive(Resource)]
pub fn derive_resource_id_macro(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_resource_id_macro(&ast)
}

#[proc_macro]
pub fn get_resource_id(input: TokenStream) -> TokenStream {
    impl_get_resource_id_macro(input)
}

#[proc_macro]
pub fn get_number_of_resources(input: TokenStream) -> TokenStream {
    impl_get_number_of_resources(input)
}

#[proc_macro]
pub fn query_resource(input: TokenStream) -> TokenStream {
    impl_query_resource(input)
}

#[proc_macro]
pub fn world_query(input: TokenStream) -> TokenStream {
    impl_world_query_macro(input)
}