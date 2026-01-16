#[macro_export]
macro_rules! include_str_from_project_path {
    ($string:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), $string))
    };
}

#[macro_export]
macro_rules! include_bytes_from_project_path {
    ($string:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), $string))
    };
}

use std::{cell::RefCell, ops::{Deref, DerefMut, Index}, rc::Rc};

pub(crate) use include_str_from_project_path;

pub struct Entry<T> {
    data: T,
    index_handle: Rc<RefCell<Option<usize>>>
}

impl<T> Entry<T> {
    pub fn get_index(&self) -> Option<usize> {
        self.index_handle.borrow().clone()
    }

    pub fn get_index_handle(&self) -> Rc<RefCell<Option<usize>>> {
        Rc::clone(&self.index_handle)
    }

    pub fn set_index(&self, index: Option<usize>) {
        let mut index_handle_mut = self.index_handle.borrow_mut();
        *index_handle_mut = index;
    }
}

impl<T> Deref for Entry<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Entry<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub struct WorldVec<T> {
    vector: Vec<Entry<T>>
}

impl<T> WorldVec<T> {
    pub fn new() -> Self {
        Self {
            vector: Vec::new()
        }
    }

    pub fn len(&self) -> usize {
        self.vector.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Entry<T>> {
        self.vector.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Entry<T>> {
        self.vector.iter_mut()
    }

    pub fn swap_remove(&mut self, index_option: Option<usize>) -> Option<T> {
        if let Some(index) = index_option {
            let vec_size = self.vector.len();
            let removed = self.vector.swap_remove(index);
            if index != vec_size - 1 {
                self.vector[index].set_index(Some(index));
            }
            removed.set_index(None);
            return Some(removed.data)
        }

        println!("Attempting to remove object which has already been removed!");
        None
    }

    pub fn push(&mut self, object: T) {
        let index = self.vector.len();
        self.vector.push(Entry { data: object, index_handle: Rc::new(RefCell::new(Some(index))) });
    }
}

impl<T> Index<usize> for WorldVec<T> {
    type Output = Entry<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vector[index]
    }
}

impl <'a, T> IntoIterator for &'a WorldVec<T> {
    type Item = &'a Entry<T>;

    type IntoIter = std::slice::Iter<'a, Entry<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.vector.iter()
    }
}

impl <'a, T> IntoIterator for &'a mut WorldVec<T> {
    type Item = &'a mut Entry<T>;

    type IntoIter = std::slice::IterMut<'a, Entry<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.vector.iter_mut()
    }
}

impl<T> IntoIterator for WorldVec<T> {
    type Item = Entry<T>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.vector.into_iter()
    }
}