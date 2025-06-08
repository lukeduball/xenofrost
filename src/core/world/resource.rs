use std::{cell::{Ref, RefCell, RefMut}, rc::Rc};

pub struct ResourceHandle<T: Resource> {
    handle: Rc<RefCell<T>>
}

impl<T: Resource> ResourceHandle<T> {
    pub fn new(reference: Rc<RefCell<dyn Resource>>) -> Self {
        unsafe {
            let ptr = Rc::into_raw(reference) as *const RefCell<T>;
            let handle = Rc::from_raw(ptr);

            ResourceHandle {
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

pub trait Resource {
    fn get_resource_id(&self) -> u64;
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};
    
    use super::{Resource, ResourceHandle};


    #[derive(Resource, Debug)]
    struct ResourceTest(u64);

    #[test]
    fn test_resource_handle_creation() {
        let test_resource = ResourceTest(98342);
        let test_resource_ref: Rc<RefCell<dyn Resource>> = Rc::new(RefCell::new(test_resource));

        let resource_handle: ResourceHandle<ResourceTest> = ResourceHandle::new(Rc::clone(&test_resource_ref));
        assert_eq!(Rc::strong_count(&resource_handle.handle), 2);

        drop(resource_handle);
        assert_eq!(Rc::strong_count(&test_resource_ref), 1);
    }

    #[test]
    fn test_resource_data() {
        let test_resource = ResourceTest(843242);
        let test_resource_ref: Rc<RefCell<dyn Resource>> = Rc::new(RefCell::new(test_resource));

        let resource_handle: ResourceHandle<ResourceTest> = ResourceHandle::new(Rc::clone(&test_resource_ref));

        assert_eq!(resource_handle.data().0, 843242);
    }

    #[test]
    fn test_resource_data_mut() {
        let test_resource = ResourceTest(843242);
        let test_resource_ref: Rc<RefCell<dyn Resource>> = Rc::new(RefCell::new(test_resource));

        let resource_handle: ResourceHandle<ResourceTest> = ResourceHandle::new(Rc::clone(&test_resource_ref));
        let mut resource_data = resource_handle.data_mut();
        assert_eq!(resource_data.0, 843242);

        resource_data.0 = 243;
        assert_eq!(resource_data.0, 243);
    }
}

pub use xenofrost_macros::Resource;