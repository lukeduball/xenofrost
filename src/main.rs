use std::rc::Rc;

use glam::Vec3;
use xenofrost::core::{engine::run, world::{component::{Test1, Test2, Test3}, ref_downcast, Entity, World}};
use xenofrost_macros::world_query;

fn main() {
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

    let query4 = world_query!(Test2, Test3);
    let result4 = query4(&world);
    for (entity, test2, test3) in result4.iter() {
        println!("This is a valid query {} {} {} {}", entity.0, test2.0, test3.color, test3.position);
    }

    pollster::block_on(run());
}
