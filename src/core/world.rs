use core::fmt;
use glam::{Vec2};

pub struct Transform2d {
    pub translation: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
}

impl Transform2d {
    pub fn rotate(&mut self, rotation_offset: f32) {
        self.set_rotation(self.rotation + rotation_offset);
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        //Add 360.0 degrees so that there will be no negative degrees and the next line will keep it between 0 and 360 degrees
        self.rotation = rotation + 360.0;
        self.rotation %= 360.0;
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Entity(pub u64);

impl Into<u64> for Entity {
    fn into(self) -> u64 {
        self.0
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity {}", self.0)
    }
}

pub struct World {
    entities: Vec<Entity>,
}

impl World {
    pub fn new() -> World {
        World {
            entities: Vec::new(),
        }
    }

    pub fn update(&mut self) {

    }
}