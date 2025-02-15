use std::{any::Any, cell::{Ref, RefMut}};

pub trait Component : Any {
    fn get_component_id(&self) -> u64;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub fn component_ref_downcast<T: Component>(reference: Ref<dyn Component>) -> Ref<T> {
    Ref::map(reference, |x| x.as_any().downcast_ref::<T>().unwrap())
}

pub fn component_mut_downcast<T: Component>(reference: RefMut<dyn Component>) -> RefMut<T> {
    RefMut::map(reference, |x| x.as_any_mut().downcast_mut::<T>().unwrap())
}