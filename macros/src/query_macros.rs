use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote,ToTokens};
use syn::{parse::{Parse, ParseStream}, parse_macro_input, Token};

use crate::macro_utilities::parse_one_or_more;

#[derive(Clone)]
struct ComponentInfo {
    component_type: syn::Type,
    _mutable: bool
}

impl Parse for ComponentInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut is_mut = false;
        if input.peek(Token![mut]) {
            _ = input.parse::<Token![mut]>();
            is_mut = true;
        }

        let item = input.parse()?;
        Ok(ComponentInfo {
            component_type: item,
            _mutable: is_mut
        })
    }
}

impl ToTokens for ComponentInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let component_type = &self.component_type;
        let result = quote! {
            #component_type
        };

        tokens.extend(result)
    }
}

struct QueryResult {
    component_types: Vec<ComponentInfo>,
    component_type_list: Vec<syn::Type>
}

impl Parse for QueryResult {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let component_types = parse_one_or_more::<ComponentInfo>(input)?;

        let mut component_type_list = Vec::new();
        for component in &component_types {
            component_type_list.push(component.component_type.clone());
        }

        Ok(Self {
            component_types,
            component_type_list
        })
    }
}

impl ToTokens for QueryResult {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let component_types = &self.component_types;
        let component_type_list = &self.component_type_list;
        let number_of_types = component_type_list.len();

        let index = (0..number_of_types).map(syn::Index::from);

        let component_ids = quote! {
            vec![#((<#component_type_list>::COMPONENT_ID)),*]
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

                    fn get_entry(&self, i: usize) -> (xenofrost::core::world::Entity #(, xenofrost::core::world::component::ComponentHandle<#component_types>)*) {
                        let entity = self.entries[i].entity;
                        let components = &self.entries[i].components;
                        return (entity #(, xenofrost::core::world::component::ComponentHandle::<#component_types>::new(components[#index].clone()))*)
                    }
                };

                struct QueryResultIterator<'a> {
                    query_result: &'a QueryResult,
                    index: usize
                };

                impl<'a> Iterator for QueryResultIterator<'a> {
                    type Item = (xenofrost::core::world::Entity #(, xenofrost::core::world::component::ComponentHandle<#component_types>)*);
                    
                    fn next(&mut self) -> Option<Self::Item> {
                        if self.index < self.query_result.entries.len() {
                            let result = self.query_result.get_entry(self.index);
                            self.index += 1;
                            Some(result)
                        }
                        else {
                            None
                        }
                    }
                };
    
                let mut entities: Vec<xenofrost::core::world::Entity> = Vec::new();
                let mut first_search = true;
                for component_id in #component_ids {
                    entities = world.get_entities_with_component(&entities, component_id, first_search);
                    first_search = false;
                }
    
                let mut query_result = QueryResult::new();
                for entity in entities {
                    let result = QueryResultEntry {
                        entity, 
                        components: [
                            #(world.query_component(entity, (<#component_type_list>::COMPONENT_ID)).unwrap(),)*
                        ]
                    };
                    query_result.entries.push(result);
                }
    
                query_result
            }
        });
    }
}

pub fn impl_world_query_macro(input: TokenStream) -> TokenStream {
    let query_result = parse_macro_input!(input as QueryResult);

    let result = quote! {
        #query_result
    };

    TokenStream::from(result)
}

