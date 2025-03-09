use std::{ops::Range, sync::Arc};

use camera::Camera;
use glam::Vec3;
use mesh::Mesh;
use winit::window::Window;
use xenofrost_macros::{query_resource, world_query, Resource};
use crate::core::world::{resource::Resource, Transform2D};

use super::world::World;

pub mod camera;
pub mod texture;
pub mod mesh;
pub mod pipeline;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
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
pub struct PrimaryRenderPass<'a> {
    pub render_pass: Option<wgpu::RenderPass<'a>>
}

#[derive(Resource)]
pub struct RenderEngine<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    window_width: u32,
    window_height: u32
}

impl<'a> RenderEngine<'a> {
    pub async fn new(window: Arc<Window>, world: &mut World, width: u32, height: u32) -> RenderEngine<'a> {
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

        //Add any applicable shared resources related to the render engine
        world.add_resource(PrimaryRenderPass {render_pass: None} );

        world.add_resource(AspectRatio {
            aspect_ratio: width as f32 / height as f32
        });

        RenderEngine {
            surface,
            device,
            queue,
            config,
            window_width: width,
            window_height: height
        }
    }

    pub fn render_frame_setup(&self, world: &mut World) -> Result<(wgpu::SurfaceTexture, wgpu::CommandEncoder), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let primary_render_pass = query_resource!(world, PrimaryRenderPass).unwrap();

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

        primary_render_pass.data_mut().render_pass = Some(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        }));

        Ok((output, encoder))
    }

    pub fn render_frame_present(&self, world: &mut World, output: wgpu::SurfaceTexture, encoder: wgpu::CommandEncoder) {
        let primary_render_pass = query_resource!(world, PrimaryRenderPass).unwrap();

        //Drop the render pass so that the render pass can be completed and used
        primary_render_pass.data_mut().render_pass = None;
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn recover_window(&mut self, world: &mut World) {
        self.resize(world, self.window_width, self.window_height);
    }

    pub fn resize(&mut self, world: &mut World, new_width: u32, new_height: u32) {
        let aspect_ratio = query_resource!(world, AspectRatio).unwrap();
            
        self.window_width = new_width;
        self.window_height = new_height;
        self.config.width = new_width;
        self.config.height = new_height;
        aspect_ratio.data_mut().aspect_ratio = new_width as f32 / new_height as f32;
        self.surface.configure(&self.device, &self.config);

        let camera_query = world_query!(Transform2D, mut Camera);
        for (_, transform2d, mut camera) in camera_query(world).iter() {
            camera.update_aspect_ratio(aspect_ratio.data().aspect_ratio);
            camera.update_uniform_buffer(
                Vec3::new(transform2d.translation.x, transform2d.translation.y, -1.0),
                Vec3::new(0.0, 0.0, 1.0),
                &self.queue
            );
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