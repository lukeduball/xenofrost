use std::sync::Arc;

use glam::Vec3;
use winit::{application::ApplicationHandler, event::{ElementState, KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::WindowId};
use xenofrost_macros::{query_resource, world_query};

use crate::core::{render_engine::camera::Camera, world::Transform2D};

use super::{input_manager::InputManager, render_engine::{AspectRatio, PrimaryRenderPass, RenderEngine}, world::World};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

pub struct App {
    window: Option<Arc<winit::window::Window>>,
    world: World,

    startup_systems: Vec<Box<dyn Fn(&mut World)>>,
    update_systems: Vec<Box<dyn Fn(&mut World)>>,
    prepare_systems: Vec<Box<dyn Fn(&mut World)>>,
    render_systems: Vec<Box<dyn Fn(&mut World)>>,
    is_startup: bool
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            world: World::new(),
            startup_systems: Vec::new(),
            update_systems: Vec::new(),
            prepare_systems: Vec::new(),
            render_systems: Vec::new(),
            is_startup: true
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut app_runner = AppRunner::new(self);
        _ = event_loop.run_app(&mut app_runner);
    }

    fn update(&mut self) {
        if self.is_startup {
            for startup_system in self.startup_systems.iter() {
                startup_system(&mut self.world);
            }
            self.is_startup = false;
        }

        for update_system in self.update_systems.iter() {
            update_system(&mut self.world);
        }

        for prepare_system in self.prepare_systems.iter() {
            prepare_system(&mut self.world);
        }
    }

    pub fn render(&mut self) -> bool {
        
        match self.render_world()
        {
            Ok(_) => {},
            //TODO capture the current window size somewhere to resize
            Err(wgpu::SurfaceError::Lost) => todo!(),//self.resize(100, 100),
            Err(wgpu::SurfaceError::OutOfMemory) => return false,
            Err(e) => eprintln!("{:?}", e),
        }

        return true
    }

    fn render_world(&mut self) -> Result<(), wgpu::SurfaceError> {
        let world = &mut self.world;
        let render_engine = query_resource!(world, RenderEngine).unwrap();

        let output = render_engine.data().surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let primary_render_pass = query_resource!(world, PrimaryRenderPass).unwrap();

        let mut encoder = render_engine.data().device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

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

        for render_system in self.render_systems.iter() {
            render_system(world);
        }

        //Drop the render pass so that the render pass can be completed and used
        primary_render_pass.data_mut().render_pass = None;
        render_engine.data().queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        let world = &mut self.world;
        let render_engine = query_resource!(world, RenderEngine).unwrap();
        let aspect_ratio = query_resource!(world, AspectRatio).unwrap();

        if new_width > 0 && new_height > 0 {
            render_engine.data_mut().config.width = new_width;
            render_engine.data_mut().config.height = new_height;
            aspect_ratio.data_mut().aspect_ratio = new_width as f32 / new_height as f32;
            render_engine.data().surface.configure(&render_engine.data().device, &render_engine.data().config);

            let camera_query = world_query!(Transform2D, mut Camera);
            for (_, transform2d, mut camera) in camera_query(world).iter() {
                camera.update_aspect_ratio(aspect_ratio.data().aspect_ratio);
                camera.update_uniform_buffer(
                    Vec3::new(transform2d.translation.x, transform2d.translation.y, -1.0),
                    Vec3::new(0.0, 0.0, 1.0),
                    &render_engine.data().queue
                );
            }
        }
    }

    pub fn add_startup_system(&mut self, function: Box<dyn Fn(&mut World)>) {
        self.startup_systems.push(function);
    }

    pub fn add_update_system(&mut self, function: Box<dyn Fn(&mut World)>) {
        self.update_systems.push(function);
    }

    pub fn add_prepare_system(&mut self, function: Box<dyn Fn(&mut World)>) {
        self.prepare_systems.push(function);
    }

    pub fn add_render_system(&mut self, function: Box<dyn Fn(&mut World)>) {
        self.render_systems.push(function);
    }

    pub fn register_app_extension<T: AppExtension>(&mut self, extension: T) {
        extension.build(self);
    }

}

struct AppRunner<'a> {
    app: &'a mut App
}

impl<'a> AppRunner<'a> {
    fn new(app: &'a mut App) -> Self {
        Self {
            app
        }
    }
}

impl<'a> ApplicationHandler for AppRunner<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(winit::window::Window::default_attributes()).unwrap();
        let size = window.inner_size();
        
        self.app.world.add_resource(AspectRatio {
            aspect_ratio: size.width as f32 / size.height as f32
        });

        let arc_window = Arc::new(window);
        self.app.world.add_resource(pollster::block_on(RenderEngine::new(Arc::clone(&arc_window), size.width, size.height)));
        self.app.world.add_resource(PrimaryRenderPass {render_pass: None} );
        self.app.world.add_resource(InputManager::new());

        self.app.window = Some(arc_window);

        #[cfg(target_arch="wasm32")]
        {
            use winit::dpi::PhysicalSize;
            let _ = window.request_inner_size(PhysicalSize::new(450, 400));
    
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm_fluid_2d")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
            })
            .expect("Could not append canvas to document body!");
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let world = &mut self.app.world;
        let input_manager = query_resource!(world, InputManager).unwrap();
        input_manager.data_mut().process_input(&event);

        self.app.update();

        match event {
            WindowEvent::CloseRequested | WindowEvent::KeyboardInput { 
                event: KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    ..
                }, 
                .. 
            } => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                self.app.resize(physical_size.width, physical_size.height);
            },
            WindowEvent::RedrawRequested => {
                if !self.app.render() {
                    event_loop.exit();
                }
                self.app.window.as_ref().unwrap().request_redraw();
            }
            _ => ()
        }
    }
}

pub trait AppExtension {
    fn build(&self, app: &mut App);
}