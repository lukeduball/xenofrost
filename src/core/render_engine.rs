use std::{ops::Range, sync::Arc};

use mesh::Mesh;
use winit::window::Window;

pub mod camera;
pub mod texture;
pub mod mesh;
pub mod pipeline;

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
    pub aspect_ratio: f32,
    render_hook: fn() -> Result<(), wgpu::SurfaceError>,
    resize_event_hook: Option<fn(&RenderEngine)>,
}

impl RenderEngine {
    pub async fn new(window: Arc<Window>, width: u32, height: u32, render_hook: fn()-> Result<(), wgpu::SurfaceError>) -> RenderEngine {
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
            aspect_ratio,
            render_hook,
            resize_event_hook: None,
        }
    }

    pub fn register_resize_event_hook(&mut self, hook: fn(&RenderEngine)) {
        self.resize_event_hook = Some(hook);
    }

//fn render_world(&self) -> Result<(), wgpu::SurfaceError> {
//    let (output, encoder) = self.render_frame_setup()?;
//
//    //TODO add the hook for render systems from applications
//    //for render_system in self.render_systems.iter() {
//    //    render_system(world);
//    //}
//
//    self.render_frame_present(output, encoder);
//
//    Ok(())
//}

    pub fn render(&mut self) -> bool {
        let render_result = (self.render_hook)();
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

    pub fn render_frame_present(&self, render_pass: wgpu::RenderPass, output: wgpu::SurfaceTexture, encoder: wgpu::CommandEncoder) {

        //Drop the render pass so that the render pass can be completed and used
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn recover_window(&mut self) {
        self.resize(self.window_width, self.window_height);
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.window_width = new_width;
        self.window_height = new_height;
        self.config.width = new_width;
        self.config.height = new_height;
        self.aspect_ratio = new_width as f32 / new_height as f32;
        self.surface.configure(&self.device, &self.config);

        //Call the resize hook function if its been registered with the Render Engine
        if let Some(resize_hook) = self.resize_event_hook {
            (resize_hook)(self);
        }
    }
}

pub trait DrawMesh<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh, camera_bind_group: &'a wgpu::BindGroup);
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
    
    fn draw_mesh(&mut self, mesh: &'b Mesh, camera_bind_group: &'b wgpu::BindGroup) {
        self.draw_mesh_instanced(mesh, 0..1, camera_bind_group);
    }
}

pub use wgpu;