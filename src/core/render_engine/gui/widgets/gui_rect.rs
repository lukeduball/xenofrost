use glam::{Vec2, Vec3};

use crate::core::render_engine::{RenderEngine, gui::{GuiAttributes, GuiElement, GuiElementInstance, GuiValue}};

pub struct GuiRect {
    attributes: GuiAttributes
}

impl GuiElement for GuiRect {
    fn get_gui_attributes(&self) -> GuiAttributes {
        self.attributes
    }

    fn generate_gui_element_instance(&self, parent_left: f32, parent_top: f32, parent_width: f32, parent_height: f32, render_engine: &RenderEngine) -> GuiElementInstance {
        let left = parent_left + self.attributes.left.convert_to_logical(parent_width);
        let top = parent_top + self.attributes.top.convert_to_logical(parent_height);
        let width = self.attributes.width.convert_to_logical(parent_width);
        let height = self.attributes.height.convert_to_logical(parent_height);

        GuiElementInstance { 
            position: render_engine.convert_coordinates_to_view_space(Vec2::new(left, top)), 
            size: render_engine.convert_extents_to_view_space(Vec2::new(width, height)), 
            color: self.attributes.color 
        }
    }
}

impl GuiRect {
    pub fn new(left: GuiValue, top: GuiValue, width: GuiValue, height: GuiValue, color: Vec3) -> Self {
        let attributes = GuiAttributes {
            left,
            top,
            width,
            height,
            color
        };
        
        Self {
            attributes
        }
    }
}