use std::sync::Arc;

use winit::{application::ApplicationHandler, event::{ElementState, KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::WindowId};
use xenofrost_macros::query_resource;

use super::{input_manager::InputManager, render_engine::RenderEngine, world::World};

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
        let render_result = self.render_world();
        match render_result
        {
            Ok(_) => {},
            Err(wgpu::SurfaceError::Lost) => {
                let world = &mut self.world;
                let render_engine = query_resource!(world, RenderEngine).unwrap();
                render_engine.data_mut().recover_window(world);
            },
            Err(wgpu::SurfaceError::OutOfMemory) => return false,
            Err(e) => eprintln!("{:?}", e),
        }

        return true
    }

    fn render_world(&mut self) -> Result<(), wgpu::SurfaceError> {
        let world = &mut self.world;
        let render_engine = query_resource!(world, RenderEngine).unwrap();

        let (output, encoder) = render_engine.data().render_frame_setup(world)?;

        for render_system in self.render_systems.iter() {
            render_system(world);
        }

        render_engine.data().render_frame_present(world, output, encoder);

        Ok(())
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            let world = &mut self.world;
            let render_engine = query_resource!(world, RenderEngine).unwrap();

            render_engine.data_mut().resize(world, new_width, new_height);
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
    
        let arc_window = Arc::new(window);

        let render_engine = pollster::block_on(RenderEngine::new(Arc::clone(&arc_window), &mut self.app.world, size.width, size.height));
        self.app.world.add_resource(render_engine);
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