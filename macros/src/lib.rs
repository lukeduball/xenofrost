use component_macros::impl_component_id_macro;
use proc_macro::TokenStream;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, Token};
use quote::{quote, ToTokens};

use proc_macro2::TokenStream as TokenStream2;

extern crate proc_macro;

mod component_macros;

#[proc_macro_derive(Component)]
pub fn derive_component_id_macro(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_component_id_macro(&ast)
}

fn parse_one_or_more<T: Parse>(input: ParseStream) -> syn::Result<Vec<T>> {
    let mut result = Vec::new();
    result.push(input.parse()?);
    while let Ok(_comma) = input.parse::<Token![,]>() {
        let item = input.parse()?;
        result.push(item);
    }

    Ok(result)
}

struct QueryResult {
    component_types: Vec<syn::Type>
}

impl Parse for QueryResult {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let component_types = parse_one_or_more(input)?;

        Ok(Self {
            component_types
        })
    }
}

impl ToTokens for QueryResult {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let component_types = &self.component_types;
        let number_of_types = component_types.len();

        let index = (0..number_of_types).map(syn::Index::from);

        let component_ids = quote! {
            vec![#((<#component_types>::COMPONENT_ID) ,)*]
        };

        tokens.extend(quote! {
            | world: &xenofrost::core::world::World | {
                struct QueryResultEntry {
                    entity: xenofrost::core::world::Entity,
                    components: [std::rc::Rc<std::cell::RefCell<dyn xenofrost::core::world::component::Component>>; #number_of_types]
                }
    
                struct QueryResult {
                    entries: Vec<QueryResultEntry>,
                };
    
                impl QueryResult {
                    fn new() -> Self {
                        let entries: Vec<QueryResultEntry> = Vec::new();
    
                        Self {
                            entries
                        }
                    }

                    fn iter(&self) -> QueryResultIterator {
                        QueryResultIterator {
                            query_result: self,
                            index: 0
                        }
                    }
                };

                struct QueryResultIterator<'a> {
                    query_result: &'a QueryResult,
                    index: usize
                };

                impl<'a> Iterator for QueryResultIterator<'a> {
                    type Item = (xenofrost::core::world::Entity #(, std::cell::Ref<'a, (#component_types)>)*);
                    
                    fn next(&mut self) -> Option<Self::Item> {
                        if self.index < self.query_result.entries.len() {
                            let query_result_entry = &self.query_result.entries[self.index];
                            let entity = query_result_entry.entity;
                            let components = &query_result_entry.components;
                            let result = (entity #(, xenofrost::core::world::ref_downcast::<#component_types>(components[#index].borrow()))*);
                            self.index += 1;
                            Some(result)
                        }
                        else {
                            None
                        }
                    }
                };
    
                let mut entities: Vec<xenofrost::core::world::Entity> = Vec::new();
                for component_id in #component_ids {
                    entities = world.get_entities_with_component(&entities, component_id);
                }
    
                let mut query_result = QueryResult::new();
                for entity in entities {
                    let result = QueryResultEntry {
                        entity, 
                        components: [
                            #(world.query_component(entity, (<#component_types>::COMPONENT_ID)).unwrap(),)*
                        ]
                    };
                    query_result.entries.push(result);
                }
    
                query_result
            }
        });
    }
}

#[proc_macro]
pub fn world_query(input: TokenStream) -> TokenStream {
    let query_result = parse_macro_input!(input as QueryResult);

    let result = quote! {
        #query_result
    };

    TokenStream::from(result)
}