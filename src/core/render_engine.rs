use std::{ops::Range, sync::Arc};

use glam::Vec2;
use mesh::Mesh;
use winit::window::Window;

pub mod buffer;
pub mod gui;
pub mod mesh;
pub mod pipeline;
pub mod render_camera;
pub mod texture;

pub fn create_command_encoder(device: &wgpu::Device, label: &str) -> wgpu::CommandEncoder {
    let command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(label)
    });

    command_encoder
}

pub struct RenderEngine {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub window_width: u32,
    pub window_height: u32,
    pub window_scale_factor: f32,
    pub window_logical_width: f32,
    pub window_logical_height: f32,
    pub aspect_ratio: f32,
}

impl RenderEngine {
    pub async fn new(window: Arc<Window>, width: u32, height: u32, scale_factor: f32) -> RenderEngine {
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
            width: width,
            height: height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let aspect_ratio = width as f32 / height as f32;

        RenderEngine {
            surface,
            device,
            queue,
            config,
            window_width: width,
            window_height: height,
            window_scale_factor: scale_factor,
            window_logical_width: width as f32 / scale_factor,
            window_logical_height: height as f32 / scale_factor,
            aspect_ratio,
        }
    }

    pub fn successful_render(&mut self, render_result: Result<(), wgpu::SurfaceError>) -> bool {
        match render_result
        {
            Ok(_) => {},
            Err(wgpu::SurfaceError::Lost) => {
                self.recover_window();
            },
            Err(wgpu::SurfaceError::OutOfMemory) => return false,
            Err(e) => eprintln!("{:?}", e),
        }

        return true
    }

    pub fn render_frame_setup<'a>(&self, encoder: &'a mut wgpu::CommandEncoder) -> Result<(wgpu::RenderPass<'a>, wgpu::SurfaceTexture), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        });

        Ok((render_pass, output))
    }

    pub fn render_frame_present(&self, output: wgpu::SurfaceTexture, encoder: wgpu::CommandEncoder) {
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn recover_window(&mut self) {
        self.resize(self.window_width, self.window_height);
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.window_width = new_width;
        self.window_height = new_height;
        self.window_logical_width = new_width as f32 / self.window_scale_factor;
        self.window_logical_height = new_height as f32 / self.window_scale_factor;
        self.config.width = new_width;
        self.config.height = new_height;
        self.aspect_ratio = new_width as f32 / new_height as f32;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn convert_coordinates_to_view_space(&self, coordinates: Vec2) -> Vec2 {
        Vec2::new(
            self.convert_x_axis_value_to_view_space(coordinates.x),
            self.convert_y_axis_value_to_view_space(coordinates.y)
        )
    }

    pub fn convert_x_axis_value_to_view_space(&self, x_value: f32) -> f32 {
        convert_coordinate_to_view_space(x_value, self.window_width as f32, self.window_scale_factor)
    }

    pub fn convert_y_axis_value_to_view_space(&self, y_value: f32) -> f32 {
        // Multiply result by negative 1 because y view space coordinates are 1(top of screen) -> -1(bottom of screen)
        -convert_coordinate_to_view_space(y_value, self.window_height as f32, self.window_scale_factor)
    }

    pub fn convert_extents_to_view_space(&self, coordinates: Vec2) -> Vec2 {
        Vec2::new(
            self.convert_width_value_to_view_space(coordinates.x),
            self.convert_height_value_to_view_space(coordinates.y)
        )
    }

    pub fn convert_width_value_to_view_space(&self, x_value: f32) -> f32 {
        convert_extent_to_view_space(x_value, self.window_width as f32, self.window_scale_factor)
    }

    pub fn convert_height_value_to_view_space(&self, y_value: f32) -> f32 {
        convert_extent_to_view_space(y_value, self.window_height as f32, self.window_scale_factor)
    }
}

pub fn convert_coordinate_to_view_space(value: f32, screen_extent: f32, screen_scaling: f32) -> f32 {
    (convert_extent_to_view_space(value, screen_extent, screen_scaling)) - 1.0
}

pub fn convert_extent_to_view_space(value: f32, screen_extent: f32, screen_scaling: f32) -> f32 {
    2.0 * (value * screen_scaling) / screen_extent
}

pub trait DrawMesh<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh, camera_bind_group: &'a wgpu::BindGroup);
    fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: Range<u32>, camera_bind_group: &'a wgpu::BindGroup);
    fn draw_mesh_no_camera(&mut self, mesh: &'a Mesh);
    fn draw_mesh_instanced_no_camera(&mut self, mesh: &'a Mesh, instances: Range<u32>);
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
    
    fn draw_mesh(&mut self, mesh: &'b Mesh, camera_bind_group: &'b wgpu::BindGroup) {
        self.draw_mesh_instanced(mesh, 0..1, camera_bind_group);
    }
    
    fn draw_mesh_no_camera(&mut self, mesh: &'b Mesh) {
        self.draw_mesh_instanced_no_camera(mesh, 0..1);
    }
    
    fn draw_mesh_instanced_no_camera(&mut self, mesh: &'b Mesh, instances: Range<u32>) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}

pub use bytemuck;
pub use wgpu;