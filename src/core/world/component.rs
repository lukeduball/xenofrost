use std::{cell::{Ref, RefCell, RefMut}, rc::Rc};

pub trait Component {
    fn get_component_id(&self) -> u64;
}

pub struct ComponentHandle<T: Component> {
    handle: Rc<RefCell<T>>
}

impl<T: Component> ComponentHandle<T> {
    pub fn new(reference: Rc<RefCell<dyn Component>>) -> Self {
        unsafe {
            let ptr = Rc::into_raw(reference) as *const RefCell<T>;
            let handle = Rc::from_raw(ptr);

            ComponentHandle { 
                handle 
            }
        }
    }

    pub fn data(&self) -> Ref<T> {
        self.handle.borrow()
    }

    pub fn data_mut(&self) -> RefMut<T> {
        self.handle.borrow_mut()
    }
}

pub use xenofrost_macros::Component;