use std::any::Any;

use glam::Vec3;
use xenofrost_macros::Component;

pub trait Component : Any {
    fn get_component_id(&self) -> u64;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Component)]
pub struct Test1(pub u64);
#[derive(Component)]
pub struct Test2(pub f64);

#[derive(Component)]
pub struct Test3 {
    pub color: Vec3,
    pub position: Vec3
}