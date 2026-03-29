use glam::{Vec2, Vec3};
use wgpu::vertex_attr_array;

use crate::{core::render_engine::{RenderEngine, buffer::VecBuffer, pipeline::{PipelineLayoutDescriptor, VertexState, create_default_pipeline2d_descriptor, create_render_pipeline_from_descriptor, create_shader}}, include_str_from_project_path};

pub mod containers;
pub mod font_renderer;
pub mod widgets;

pub struct GuiManager {
    gui_list: Vec<Box<dyn GuiElement>>,
}

impl GuiManager {
    pub fn new() -> Self {
        Self {
            gui_list: Vec::new()
        }
    }

    pub fn add_gui(&mut self, gui: Box<dyn GuiElement>) {
        self.gui_list.push(gui);
    }

    // Need to provide the GuiElement handle to remove
    pub fn remove_gui(&mut self) {

    }
 }

pub struct GuiRenderer {
    gui_pipeline: wgpu::RenderPipeline,
    gui_element_instance_list: VecBuffer<GuiElementInstance>
}

impl GuiRenderer {
    pub fn new(
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        aspect_ratio_bind_group_layout: &wgpu::BindGroupLayout
    ) -> Self {
        Self {
            gui_pipeline: create_gui_pipeline(device, config, texture_bind_group_layout, aspect_ratio_bind_group_layout),
            gui_element_instance_list: VecBuffer::new(device, "Gui Instances", wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST)
        }
    }

    pub fn prepare_gui_render_instances(&mut self, manager: &GuiManager, render_engine: &RenderEngine) {
        self.gui_element_instance_list.clear();
        for gui_element in &manager.gui_list {
            // At the highest level the "parent" is the window itself. I.e. x and y of 0.0 with the logical width and height
            let gui_render_instance = gui_element.generate_gui_element_instance(0.0, 0.0, render_engine.window_logical_width, render_engine.window_logical_height, render_engine);
            self.gui_element_instance_list.push(gui_render_instance);
        }
        self.gui_element_instance_list.update_buffer_data(&render_engine.device, &render_engine.queue);
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass, texture_atlas_bind_group: &wgpu::BindGroup, aspect_ratio_bind_group: &wgpu::BindGroup) {
        render_pass.set_pipeline(&self.gui_pipeline);
        render_pass.set_vertex_buffer(0, self.gui_element_instance_list.get_buffer().slice(..));
        render_pass.set_bind_group(0, texture_atlas_bind_group, &[]);
        render_pass.set_bind_group(1, aspect_ratio_bind_group, &[]);
        // There are 6 indices that need to be drawn
        render_pass.draw(0..6, 0..self.gui_element_instance_list.len() as u32);
    }
}

#[derive(Clone, Copy)]
pub enum GuiValue {
    Pixels(f32),
    Percent(f32)
}

impl GuiValue {
    pub fn convert_to_logical(&self, parent_value: f32) -> f32 {
        match self {
            GuiValue::Pixels(px) => *px,
            GuiValue::Percent(pct) => parent_value * pct,
        }
    }
}

#[derive(Clone, Copy)]
pub struct GuiAttributes {
    left: GuiValue,
    top: GuiValue,
    width: GuiValue,
    height: GuiValue,
    color: Vec3
}

pub trait GuiElement {
    fn get_gui_attributes(&self) -> GuiAttributes;
    fn generate_gui_element_instance(&self, parent_left: f32, parent_top: f32, parent_width: f32, parent_height: f32, render_engine: &RenderEngine) -> GuiElementInstance;
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GuiElementInstance {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Vec3,
}

impl GuiElementInstance {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x3,
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES
        }
    }
}

fn create_gui_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    aspect_ratio_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let pipeline_layout_descriptor = PipelineLayoutDescriptor {
        label: "GUI Pipeline",
        bind_group_layouts: vec![texture_bind_group_layout, aspect_ratio_bind_group_layout]
    };
    let shader_module = create_shader(device, "GUI Shader", include_str_from_project_path!("/res/shaders/gui.wgsl"));
    let mut pipeline_descriptor = create_default_pipeline2d_descriptor(config, &pipeline_layout_descriptor, &shader_module);
    pipeline_descriptor.label = "SDF Font Pipeline";
    pipeline_descriptor.vertex = VertexState { module: &shader_module, entry_point: "vs_main", buffers: vec![GuiElementInstance::desc()] };

    let pipeline = create_render_pipeline_from_descriptor(device, pipeline_descriptor);
    pipeline
}