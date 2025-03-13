use std::sync::atomic::{AtomicU64, Ordering};
use quote::{quote, ToTokens};

use proc_macro::TokenStream;
use syn::{parse::Parse, parse_macro_input, Token};
use proc_macro2::TokenStream as TokenStream2;

static RESOURCE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn impl_resource_id_macro(ast: &syn::DeriveInput) -> TokenStream {
    let id = RESOURCE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let gen = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub const RESOURCE_ID: u64 = crate::BASELINE_NUMBER_OF_RESOURCES + #id;
        }

        impl #impl_generics Resource for #name #ty_generics #where_clause {
            fn get_resource_id(&self) -> u64 {
                crate::BASELINE_NUMBER_OF_RESOURCES + #id
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

struct ResourceQuery {
    world: syn::Ident,
    resource_type: syn::Type
}

impl Parse for ResourceQuery {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let world = input.parse()?;
        _ = input.parse::<Token![,]>();
        let resource_type = input.parse()?;
        
        Ok(Self {
            world,
            resource_type
        })
    }
}

impl ToTokens for ResourceQuery {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let world = &self.world;
        let resource_type = &self.resource_type;

        let result = quote! {
            #world.query_resource::<#resource_type>(<#resource_type>::RESOURCE_ID)
        };

        tokens.extend(result);
    }
}

pub fn impl_query_resource(input: TokenStream) -> TokenStream {
    let resource_query = parse_macro_input!(input as ResourceQuery);

    let result = quote! {
        #resource_query
    };

    TokenStream::from(result)
}

pub fn impl_get_number_of_resources(_input: TokenStream) -> TokenStream {
    let number_of_resources = RESOURCE_ID_COUNTER.load(Ordering::SeqCst);

    let result = quote! {
        #number_of_resources
    };

    TokenStream::from(result)
}