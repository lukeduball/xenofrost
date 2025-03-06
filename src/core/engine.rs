use std::sync::Arc;

use cfg_if::cfg_if;
use pollster::block_on;
use winit::{application::ApplicationHandler, event::{ElementState, KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use xenofrost_macros::get_resource_id;

use crate::core::{input_manager::InputManager, render_engine::RenderEngine, world::{World, WorldHandler}};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

use super::render_engine::AspectRatio;


struct Engine {
    window: Option<Arc<Window>>,
    world_handler: WorldHandler,
    world: World,
}

impl Engine {
    fn new() -> Self {
        Self {
            window: None,
            world_handler: WorldHandler::new(),
            world: World::new(),
        }
    }
}

impl ApplicationHandler for Engine {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        let size = window.inner_size();

        self.world.add_resource(AspectRatio {
            aspect_ratio: size.width as f32 / size.height as f32
        });

        let arc_window = Arc::new(window);
        self.world.add_resource(block_on(RenderEngine::new(Arc::clone(&arc_window), size.width, size.height)));
        self.world.add_resource(InputManager::new());
        self.world_handler.initialize(&mut self.world);

        self.window = Some(arc_window);

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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let input_manager = self.world.query_resource::<InputManager>(get_resource_id!(InputManager)).unwrap();
        //TODO This could potentially be added as a system instead
        input_manager.data_mut().process_input(&event);

        self.world_handler.update(&mut self.world);

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
                self.world_handler.resize(physical_size.width, physical_size.height, &mut self.world);
            },
            WindowEvent::RedrawRequested => {
                if !self.world_handler.render(&mut self.world) {
                    event_loop.exit();
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => ()
        }
    }
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if!(
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Unable to initialize logger!");
        }
        else {
            env_logger::init();
        }
    );

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut engine = Engine::new();
    _ = event_loop.run_app(&mut engine);
}