use glam::{Vec2, Vec3};
use wgpu::vertex_attr_array;

use crate::{core::render_engine::{DrawMesh, buffer::VecBuffer, mesh::{Mesh, PositionVertex, Vertex}, pipeline::{PipelineLayoutDescriptor, VertexState, create_default_pipeline2d_descriptor, create_render_pipeline_from_descriptor, create_shader}}, include_str_from_project_path};

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

    pub fn prepare_gui_render_instances(&mut self, manager: &GuiManager, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.gui_element_instance_list.clear();
        for gui_element in &manager.gui_list {
            let gui_render_instance = gui_element.generate_gui_element_instance();
            self.gui_element_instance_list.push(gui_render_instance);
        }
        self.gui_element_instance_list.update_buffer_data(device, queue);
    }

    pub fn render<'a, 'b>(&self, render_pass: &'a mut wgpu::RenderPass<'b>, quad_mesh: &'b Mesh, texture_atlas_bind_group: &wgpu::BindGroup, aspect_ratio_bind_group: &wgpu::BindGroup) {
        render_pass.set_pipeline(&self.gui_pipeline);
        render_pass.set_vertex_buffer(1, self.gui_element_instance_list.get_buffer().slice(..));
        render_pass.set_bind_group(0, texture_atlas_bind_group, &[]);
        render_pass.set_bind_group(1, aspect_ratio_bind_group, &[]);
        render_pass.draw_mesh_instanced_no_camera(quad_mesh, 0..self.gui_element_instance_list.len() as u32);
    }
}

pub trait GuiElement {
    fn get_relative_position(&self) -> Vec2;
    fn get_relative_size(&self) -> Vec2;
    fn generate_gui_element_instance(&self) -> GuiElementInstance;
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
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x3,
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
    pipeline_descriptor.vertex = VertexState { module: &shader_module, entry_point: "vs_main", buffers: vec![PositionVertex::desc(), GuiElementInstance::desc()] };

    let pipeline = create_render_pipeline_from_descriptor(device, pipeline_descriptor);
    pipeline
}