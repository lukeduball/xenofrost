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

    pub fn is_empty(&self) -> bool {
        self.vector.is_empty()
    }

    pub fn len(&self) -> usize {
        self.vector.len()
    }

    pub fn iter(&'_ self) -> std::slice::Iter<'_, Entry<T>> {
        self.vector.iter()
    }

    pub fn iter_mut(&'_ mut self) -> std::slice::IterMut<'_, Entry<T>> {
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

pub fn convert_seconds_to_frames(seconds: f32) -> u32 {
    let frames = seconds * 60.0;
    frames.ceil() as u32
}

//Timer value and count are in frames. There 60 frames a second.
pub struct Timer {
    count: u32,
    expire_time: u32,
}

impl Timer {
    pub fn create_timer_with_seconds(seconds: f32) -> Self {
        let timer_frames = convert_seconds_to_frames(seconds);
        Self::create_timer_from_update_frames(timer_frames)
    }

    pub fn create_timer_from_update_frames(frames: u32) -> Self {
        Self {
            count: 0,
            expire_time: frames
        }
    }

    pub fn set_expire_time_from_seconds(&mut self, expire_time_sec: f32) {
        let timer_frames = convert_seconds_to_frames(expire_time_sec);
        self.set_expire_time_from_frames(timer_frames);
    }

    pub fn set_expire_time_from_frames(&mut self, expire_time: u32) {
        self.expire_time = expire_time;
    }

    pub fn run(&mut self) {
        if !self.is_timer_expired() {
            self.count += 1;
        }
    }

    pub fn is_timer_expired(&self) -> bool {
        self.count >= self.expire_time
    }

    pub fn initialize_timer(&mut self) {
        self.count = 0;
    }
}