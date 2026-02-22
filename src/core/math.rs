pub mod bounding2d;

pub use glam::*;

#[derive(Clone)]
pub struct Transform2d {
    translation: Vec2,
    rotation: f32,
    scale: Vec2
}

impl Transform2d {
    pub fn new(translation: Vec2, rotation: f32, scale: Vec2) -> Self {
        Transform2d { translation, rotation, scale }
    }

    pub fn rotate(&mut self, rotation_offset: f32) {
        self.set_rotation(self.rotation + rotation_offset);
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        //Add 360.0 degrees so that there will be no negative degrees and the next line will keep it between 0 and 360 degrees
        self.rotation = rotation + 360.0;
        self.rotation %= 360.0;
    }

    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }

    pub fn translate(&mut self, translation_offset: Vec2) {
        self.set_translation(self.translation + translation_offset);
    }

    pub fn set_translation(&mut self, translation: Vec2) {
        self.translation = translation;
    }

    pub fn get_translation(&self) -> Vec2 {
        self.translation
    }

    pub fn scale(&mut self, scale_offset: Vec2) {
        self.set_scale(self.scale + scale_offset);
    }

    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
    }

    pub fn get_scale(&self) -> Vec2 {
        self.scale
    }
}