use glam::{Vec2, Vec3};

use crate::core::render_engine::gui::{GuiElement, GuiElementInstance};

pub struct GuiRect {
    position: Vec2,
    size: Vec2,
    color: Vec3
}

impl GuiElement for GuiRect {
    fn get_relative_position(&self) -> glam::Vec2 {
        self.position
    }

    fn get_relative_size(&self) -> glam::Vec2 {
        self.size
    }

    fn generate_gui_element_instance(&self) -> GuiElementInstance {
        GuiElementInstance { 
            position: self.position, 
            size: self.size, 
            color: self.color 
        }
    }
}

impl GuiRect {
    pub fn new(position: Vec2, size: Vec2, color: Vec3) -> Self {
        Self {
            position,
            size,
            color
        }
    }
}