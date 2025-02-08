use std::{cell::{Ref, RefCell, RefMut}, collections::HashMap, rc::Rc};
use component::Component;

pub mod component;
mod query;

type EntityComponentMap = HashMap<Entity, Rc<RefCell<dyn Component>>>;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Entity(u64);

impl Into<u64> for Entity {
    fn into(self) -> u64 {
        self.0
    }
}

pub fn borrow_downcast<T: Component>(cell: &RefCell<dyn Component>) -> Ref<T> {
    let r = cell.borrow();
    Ref::map(r, |x| x.as_any().downcast_ref::<T>().unwrap())
}

pub fn ref_downcast<T: Component>(reference: Ref<dyn Component>) -> Ref<T> {
    Ref::map(reference, |x| x.as_any().downcast_ref::<T>().unwrap())
}

pub fn borrow_mut_downcast<T: Component>(cell: &RefCell<dyn Component>) -> RefMut<T> {
    let r = cell.borrow_mut();
    RefMut::map(r, |x| x.as_any_mut().downcast_mut::<T>().unwrap())
}

//Note: This struct is also used to create the Render World. If specific World only or RenderWorld only items are required
//      these should be split into two structs.
pub struct World {
    entities: Vec<Entity>,
    components: HashMap<u64, EntityComponentMap>
}

impl World {
    pub fn new() -> World {
        World {
            entities: Vec::new(),
            components: HashMap::new()
        }
    }

    pub fn update(&mut self) {

    }

    pub fn get_entities_with_component(&self, entity_list: &Vec<Entity>, component_id: u64) -> Vec<Entity> {
        let mut result_entity_list: Vec<Entity> = Vec::new();

        let component_hash_map_option = self.components.get(&component_id);
        if let Some(component_hash_map) = component_hash_map_option {
            if !entity_list.is_empty() {
                result_entity_list = self.entities.iter().cloned().filter(|entity| component_hash_map.contains_key(entity)).collect();
            }
            else {
                result_entity_list = entity_list.iter().cloned().filter(|entity| component_hash_map.contains_key(entity)).collect();
            }
        }

        result_entity_list
    }

    pub fn query_component(&self, entity: Entity, component_id: u64) -> Option<Rc<RefCell<dyn Component>>> {
        let mut result: Option<Rc<RefCell<dyn Component>>> = None;

        let component_hash_map_option = self.components.get(&component_id);
        if let Some(component_hash_map) = component_hash_map_option {
            let component_option = component_hash_map.get(&entity);
            if let Some(component) = component_option {
                result = Some(Rc::clone(component));
            }
        }

        result
    }
}