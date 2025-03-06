use std::{mem, ops::Range, sync::Arc};

use mesh::Mesh;
use winit::window::Window;
use xenofrost_macros::{Component, Resource};
use crate::core::world::{resource::Resource, component::Component};

pub mod camera;
pub mod texture;
pub mod mesh;
pub mod pipeline;

#[derive(Resource)]
pub struct RenderCircleInstances {
    pub instances: Vec<InstanceRaw>,
    pub prev_size: usize,
    pub instances_buffer: wgpu::Buffer,
}

impl RenderCircleInstances {
    pub fn new(device: &wgpu::Device) -> Self {
        let instances = Vec::new();
        let instances_buffer = device.create_buffer(&wgpu::BufferDescriptor { 
            label: Some("Circle Instances"), 
            size: 1, 
            usage: wgpu::BufferUsages::VERTEX, 
            mapped_at_creation: false 
        });

        let prev_size = instances.len();

        Self {
            instances,
            prev_size,
            instances_buffer
        }
    }
}

#[derive(Component)]
pub struct RenderCircle;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ]
        }
    }
}

#[derive(Resource)]
pub struct AspectRatio {
    pub aspect_ratio: f32
}

#[derive(Resource)]
pub struct RenderEngine<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl<'a> RenderEngine<'a> {
    pub async fn new(window: Arc<Window>, width: u32, height: u32) -> RenderEngine<'a> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch="wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch="wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
                ..Default::default()
            }, 
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 450,
            height: 400,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        RenderEngine {
            surface,
            device,
            queue,
            config,
        }
    }
}

pub trait DrawMesh<'a> {
    fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: Range<u32>, camera_bind_group: &'a wgpu::BindGroup);
}

impl<'a,'b> DrawMesh<'b> for wgpu::RenderPass<'a> 
where 'b: 'a,
{
    fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: Range<u32>, camera_bind_group: &'a wgpu::BindGroup) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.set_bind_group(0, &camera_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}