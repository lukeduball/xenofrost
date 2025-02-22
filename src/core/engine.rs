use std::sync::Arc;

use cfg_if::cfg_if;
use pollster::block_on;
use winit::{application::ApplicationHandler, event::{ElementState, Event, KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};

use crate::core::{input_manager::InputManager, render_engine::RenderEngine, world::{World, WorldHandler}};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;


struct Engine<'a> {
    window: Option<Arc<Window>>,
    world_handler: WorldHandler,
    world: World,
    render_engine: Option<RenderEngine<'a>>,
    input_manager: InputManager
}

impl<'a> Engine<'a> {
    fn new() -> Self {
        Self {
            window: None,
            world_handler: WorldHandler::new(),
            world: World::new(),
            render_engine: None,
            input_manager: InputManager::new()
        }
    }
}

impl<'a> ApplicationHandler for Engine<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        let size = window.inner_size();
        self.world_handler.initialize();

        let arc_window = Arc::new(window);
        self.render_engine = Some(block_on(RenderEngine::new(Arc::clone(&arc_window), size.width, size.height)));
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
        
        self.world_handler.update(&mut self.world);
        self.input_manager.process_input(&event);

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
                self.render_engine.as_mut().unwrap().resize(physical_size.width, physical_size.height);
            },
            WindowEvent::RedrawRequested => {
                if !self.render_engine.as_mut().unwrap().render_event() {
                    event_loop.exit();
                }
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