use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote,ToTokens};
use syn::{parse::{Parse, ParseStream}, parse_macro_input, Token};

use crate::macro_utilities::parse_one_or_more;

#[derive(Clone)]
struct ComponentInfo {
    component_type: syn::Type,
    mutable: bool
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
            mutable: is_mut
        })
    }
}

impl ToTokens for ComponentInfo {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let component_type = &self.component_type;
        let result = if self.mutable {
            quote! {
                std::cell::RefMut<'a, #component_type>
            }
        }
        else {
            quote! {
                std::cell::Ref<'a, #component_type>
            }
        };

        tokens.extend(result)
    }
}

struct BorrowTokenGen {
    mutable: bool
}

impl BorrowTokenGen {
    fn new(mutable: bool) -> Self {
        Self { mutable }
    }
}

impl ToTokens for BorrowTokenGen {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let result = if self.mutable {
            quote! {
                .borrow_mut()
            }
        }
        else {
            quote! {
                .borrow()
            }
        };

        tokens.extend(result);
    }
}

struct ComponentCastTokenGen {
    component_info: ComponentInfo
}

impl ComponentCastTokenGen {
    fn new(component_info: ComponentInfo) -> Self {
        Self { component_info }
    }
}

impl ToTokens for ComponentCastTokenGen {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let component_type = &self.component_info.component_type;

        let result = if self.component_info.mutable {
            quote! {
                crate::core::world::mut_downcast::<#component_type>
            }
        }
        else {
            quote! {
                crate::core::world::ref_downcast::<#component_type>
            }
        };

        tokens.extend(result);
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

        let mut component_type_query_const = Vec::new();
        let mut component_borrow = Vec::new();
        for component_type in component_types {
            component_type_query_const.push(ComponentCastTokenGen::new(component_type.clone()));
            component_borrow.push(BorrowTokenGen::new(component_type.mutable));
        }

        let index = (0..number_of_types).map(syn::Index::from);

        let component_ids = quote! {
            vec![#((<#component_type_list>::COMPONENT_ID)),*]
        };

        tokens.extend(quote! {
            | world: &crate::core::world::World | {
                struct QueryResultEntry {
                    entity: crate::core::world::Entity,
                    components: [std::rc::Rc<std::cell::RefCell<dyn crate::core::world::component::Component>>; #number_of_types]
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
                    type Item = (crate::core::world::Entity #(, (#component_types))*);
                    
                    fn next(&mut self) -> Option<Self::Item> {
                        if self.index < self.query_result.entries.len() {
                            let query_result_entry = &self.query_result.entries[self.index];
                            let entity = query_result_entry.entity;
                            let components = &query_result_entry.components;
                            let result = (entity #(, (#component_type_query_const)(components[#index]#component_borrow))*);
                            self.index += 1;
                            Some(result)
                        }
                        else {
                            None
                        }
                    }
                };
    
                let mut entities: Vec<crate::core::world::Entity> = Vec::new();
                for component_id in #component_ids {
                    entities = world.get_entities_with_component(&entities, component_id);
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

