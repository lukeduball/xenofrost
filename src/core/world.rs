use std::{cell::{Ref, RefCell, RefMut}, collections::HashMap, rc::Rc};
use component::Component;

pub mod component;

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

    pub fn spawn_entity(&mut self) -> Entity {
        let entity = Entity(self.entities.len() as u64);
        self.entities.push(entity);
        entity
    }

    pub fn add_component_to_entity<T: Component>(&mut self, entity: Entity, component: T) {
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
    use xenofrost_macros::{world_query, Component};

    use super::{component::Component, World};

    #[derive(Component)]
    struct Test1(u64);
    #[derive(Component)]
    struct Test2(f64);

    #[derive(Component)]
    struct Test3 {
        color: Vec3,
        position: Vec3
    }


    #[test]
    fn query_world_test() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        world.add_component_to_entity(entity1, Test1(1));

        let entity2 = world.spawn_entity();
        world.add_component_to_entity(entity2, Test1(5));
        world.add_component_to_entity(entity2, Test2(4.234));


        let entity3 = world.spawn_entity();
        world.add_component_to_entity(entity3, Test2(6.4353));
        world.add_component_to_entity(entity3, Test3 {
        color: Vec3::new(1.0, 0.5, 0.5),
        position: Vec3::new(323.0, 434.4, 934.3) 
        });


        let entity4 = world.spawn_entity();
        world.add_component_to_entity(entity4, Test1(99));
        world.add_component_to_entity(entity4, Test2(8453.34));
        world.add_component_to_entity(entity4, Test3 {
            color: Vec3::new(0.0, 0.0, 1.0),
            position: Vec3::new(342.0, 965.0, 4.0)
        });

        let query1 = world_query!(Test1, Test2, Test3);
        let result1 = query1(&world);
        for (entity, test1, test2, test3) in result1.iter() {
            println!("This is a valid query {} {} {} {} {}", entity.0, test1.0, test2.0, test3.color, test3.position);
        }

        let query2 = world_query!(Test1);
        let result2 = query2(&world);
        for (entity, test1) in result2.iter() {
            println!("This is a valid query {} {}", entity.0, test1.0);
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