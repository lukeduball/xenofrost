pub mod component;

struct Entity(u64);

struct World {
    entities: Vec<Entity>,
}

impl World {
    pub fn update(&mut self) {

    }
}