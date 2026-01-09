use glam::Vec2;

use crate::core::math::{Transform2d, bounding2d::Polygon2d};

pub mod camera;

pub trait WorldObject2d {
    fn get_transform2d(&self) -> &Transform2d;

    fn translate(&mut self, translation: Vec2);
    fn set_translation(&mut self, translation: Vec2);
    fn rotate(&mut self, rotation: f32);
    fn set_rotation(&mut self, rotation: f32);
    fn scale(&mut self, scale_factor: Vec2);
    fn set_scale(&mut self, scale_factor: Vec2);
}

pub trait WorldCollisionObject2d {
    fn get_collider(&self) -> &Polygon2d;
}