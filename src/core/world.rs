use std::{cell::RefCell, collections::HashMap, rc::Rc};
use component::Component;
use glam::Vec2;
use resource::{Resource, ResourceHandle};

pub use xenofrost_macros::{query_resource, world_query};

pub mod component;
pub mod resource;

#[derive(Component)]
pub struct Transform2D {
    pub translation: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
}

impl Transform2D {
    pub fn rotate(&mut self, rotation_offset: f32) {
        self.set_rotation(self.rotation + rotation_offset);
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        //Add 360.0 degrees so that there will be no negative degrees and the next line will keep it between 0 and 360 degrees
        self.rotation = rotation + 360.0;
        self.rotation %= 360.0;
    }
}

type EntityComponentMap = HashMap<Entity, Rc<RefCell<dyn Component>>>;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Entity(pub u64);

impl Into<u64> for Entity {
    fn into(self) -> u64 {
        self.0
    }
}

//Note: This struct is also used to create the Render World. If specific World only or RenderWorld only items are required
//      these should be split into two structs.
pub struct World {
    entities: Vec<Entity>,
    components: HashMap<u64, EntityComponentMap>,
    resources: HashMap<u64, Rc<RefCell<dyn Resource>>>,
}

impl World {
    pub fn new() -> World {
        World {
            entities: Vec::new(),
            components: HashMap::new(),
            resources: HashMap::new()
        }
    }

    pub fn spawn_entity(&mut self) -> Entity {
        let entity = Entity(self.entities.len() as u64);
        self.entities.push(entity);
        entity
    }

    pub fn add_resource<T: Resource + 'static>(&mut self, resource: T) -> &mut Self {
        let resource_id = resource.get_resource_id();
        let resource_ref: Rc<RefCell<dyn Resource>> = Rc::new(RefCell::new(resource));

        self.resources.insert(resource_id, resource_ref);

        self
    }

    pub fn query_resource<T: Resource>(&mut self, resource_id: u64) -> Option<ResourceHandle<T>> {
        let result = self.resources.get(&resource_id);
        match result {
            Some(resource) => {
                Some(ResourceHandle::new(Rc::clone(&resource)))
            },
            _ => None
        }
    }

    pub fn add_component_to_entity<T: Component>(&mut self, entity: Entity, component: T) -> &mut Self {
        let component_id = component.get_component_id();
        let component_ref: Rc<RefCell<dyn Component>> = Rc::new(RefCell::new(component));

        let component_hash_table_option = self.components.get_mut(&component_id);
        match component_hash_table_option {
            Some(component_hash_table) => {
                component_hash_table.insert(entity, component_ref);
            }
            None => {
                let mut entity_component_hash_map = HashMap::new();
                entity_component_hash_map.insert(entity, component_ref);
                self.components.insert(component_id, entity_component_hash_map);
            }
        }

        self
    }

    pub fn get_entities_with_component(&self, entity_list: &Vec<Entity>, component_id: u64) -> Vec<Entity> {
        let mut result_entity_list: Vec<Entity> = Vec::new();

        let component_hash_map_option = self.components.get(&component_id);
        if let Some(component_hash_map) = component_hash_map_option {
            if entity_list.is_empty() {
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

#[cfg(test)]
mod tests {
    use glam::Vec3;
    use super::{query_resource, world_query};
    use super::{component::Component, resource::Resource, World};

    //Required to allow query_macro to resolve types in this crate and external crates
    use crate as xenofrost;

    #[derive(Component)]
    struct Test1(u64);
    #[derive(Component)]
    struct Test2(f64);

    #[derive(Component)]
    struct Test3 {
        color: Vec3,
        position: Vec3
    }

    #[derive(Resource, Debug)]
    struct ResourceTest(u64);

    #[test]
    fn query_world_test() {
        let mut world = World::new();
        world.add_resource(ResourceTest(543));

        let resource_handle = query_resource!(world, ResourceTest).unwrap();

        let entity1 = world.spawn_entity();
        world.add_component_to_entity(entity1, Test1(1));

        let entity2 = world.spawn_entity();
        world.add_component_to_entity(entity2, Test1(5))
        .add_component_to_entity(entity2, Test2(4.234));


        let entity3 = world.spawn_entity();
        world.add_component_to_entity(entity3, Test2(6.4353))
        .add_component_to_entity(entity3, Test3 {
        color: Vec3::new(1.0, 0.5, 0.5),
        position: Vec3::new(323.0, 434.4, 934.3) 
        });


        let entity4 = world.spawn_entity();
        world.add_component_to_entity(entity4, Test1(99))
        .add_component_to_entity(entity4, Test2(8453.34))
        .add_component_to_entity(entity4, Test3 {
            color: Vec3::new(0.0, 0.0, 1.0),
            position: Vec3::new(342.0, 965.0, 4.0)
        });

        let mut resource_data = resource_handle.data_mut();
        let query1 = world_query!(Test1, Test2, Test3);
        let result1 = query1(&world);
        for (entity, test1, test2, test3) in result1.iter() {
            println!("This is a valid query {} {} {} {} {}", entity.0, test1.0, test2.0, test3.color, test3.position);
            assert_eq!(resource_data.0, 543);
        }

        resource_data.0 = 19;
        let query2 = world_query!(Test1);
        let result2 = query2(&world);
        for (entity, test1) in result2.iter() {
            println!("This is a valid query {} {}", entity.0, test1.0);
            assert_eq!(resource_data.0, 19);
        }

        let query3 = world_query!(Test1, Test2);
        let result3 = query3(&world);
        for (entity, test1, test2) in result3.iter() {
            println!("This is a valid query {} {} {}", entity.0, test1.0, test2.0);
        }

        let query_mut = world_query!(mut Test2, Test3);
        let query_mut_result = query_mut(&world);
        for (entity, mut test2, test3) in query_mut_result.iter() {
            println!("This is a mut pre-query {} {} {} {}", entity.0, test2.0, test3.color, test3.position);
            test2.0 = 10.5;
            println!("This is a mut post-query {} {} {} {}", entity.0, test2.0, test3.color, test3.position);
        }

        let query4 = world_query!(Test2, Test3);
        let result4 = query4(&world);
        for (entity, test2, test3) in result4.iter() {
            println!("This is a valid query {} {} {} {}", entity.0, test2.0, test3.color, test3.position);
        }
    }
}