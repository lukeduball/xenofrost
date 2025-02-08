use std::rc::Rc;

use xenofrost::core::{engine::run, world::{component::{Test1, Test2}, Entity, World}};
use xenofrost_macros::world_query;

fn main() {
    let world = World::new();
    let query = world_query!(Test1, Test2, Test1, Test1, Test2);
    let result = query(&world);

    for (entity, test1, test2, test3, test4, test5) in result.iter() {
        println!("This is a valid query!");
    }

    pollster::block_on(run());
}
