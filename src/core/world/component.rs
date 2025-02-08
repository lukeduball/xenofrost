use std::any::Any;

use xenofrost_macros::Component;

pub trait Component : Any {
    fn get_component_id(&self) -> u64;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Component)]
pub struct Test1;
#[derive(Component)]
pub struct Test2;