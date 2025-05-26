use std::sync::Arc;

use winit::{application::ApplicationHandler, event::{ElementState, KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy}, keyboard::{KeyCode, PhysicalKey}, window::WindowId};
use xenofrost_macros::query_resource;

use super::{input_manager::InputManager, render_engine::RenderEngine, world::World};

#[allow(dead_code)]
pub struct App {
    app_name: &'static str,
    window: Option<Arc<winit::window::Window>>,
    world: World,

    startup_systems: Vec<Box<dyn Fn(&mut World)>>,
    update_systems: Vec<Box<dyn Fn(&mut World)>>,
    prepare_systems: Vec<Box<dyn Fn(&mut World)>>,
    render_systems: Vec<Box<dyn Fn(&mut World)>>,
    is_startup: bool
}

impl App {
    pub fn new(app_name: &'static str) -> Self {
        Self {
            app_name,
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
        let event_loop = EventLoop::with_user_event().build().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut app_runner = AppRunner::new(self, &event_loop);
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

//Used by wasm32 build so warning was turned off
#[allow(dead_code)]
enum EngineEvent {
    CreateGraphicsEvent(RenderEngine)
}

//Event loop proxy is used by wasm32 so warning was turned off
#[allow(dead_code)]
struct AppRunner<'a> {
    app: &'a mut App,
    event_loop_proxy: Option<EventLoopProxy<EngineEvent>>
}

impl<'a> AppRunner<'a> {
    fn new(app: &'a mut App, event_loop: &EventLoop<EngineEvent>) -> Self {
        Self {
            app,
            event_loop_proxy: Some(event_loop.create_proxy())
        }
    }
}

impl<'a> ApplicationHandler<EngineEvent> for AppRunner<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(winit::window::Window::default_attributes()).unwrap();
        //wasm32 mutates this variable to statically set the window size
        #[allow(unused_mut)]
        let mut size = window.inner_size();
        #[cfg(target_arch="wasm32")]
        {
            use winit::dpi::PhysicalSize;
            size.width = 1280;
            size.height = 720;
            _ = window.request_inner_size(PhysicalSize::new(size.width, size.height));
    
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id(self.app.app_name)?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
            })
            .expect("Could not append canvas to document body!");
        }
    
        let arc_window = Arc::new(window);

        self.app.world.add_resource(InputManager::new());

        self.app.window = Some(Arc::clone(&arc_window));

        #[cfg(target_arch="wasm32")]
        {
            let event_loop_proxy = self.event_loop_proxy.take();
            let render_engine_fut = RenderEngine::new(Arc::clone(&arc_window), size.width, size.height);
            wasm_bindgen_futures::spawn_local(async move {
                let render_engine = render_engine_fut.await;
                assert!(event_loop_proxy.unwrap().send_event(EngineEvent::CreateGraphicsEvent(render_engine)).is_ok());
            });
        }

        #[cfg(not(target_arch="wasm32"))]
        {
            let render_engine = pollster::block_on(RenderEngine::new(arc_window, size.width, size.height));
            render_engine.initialize_additional_resources(&mut self.app.world);
            self.app.world.add_resource(render_engine);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let world = &mut self.app.world;
        let scale_factor = self.app.window.as_ref().unwrap().scale_factor();

        let render_engine = query_resource!(world, RenderEngine);
        if let None = render_engine {
            //Wait until the graphics have been created
            return;
        }

        let input_manager = query_resource!(world, InputManager).unwrap();
        input_manager.data_mut().process_input(&event, scale_factor);

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
                #[cfg(not(target_arch="wasm32"))] {
                    self.app.resize(physical_size.width, physical_size.height);
                }
            },
            WindowEvent::RedrawRequested => {
                input_manager.data_mut().process_button_press_release_data();
                self.app.update();
                
                if !self.app.render() {
                    event_loop.exit();
                }
                self.app.window.as_ref().unwrap().request_redraw();
            }
            _ => ()
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: EngineEvent) {
        match event {
            EngineEvent::CreateGraphicsEvent(render_engine) => {
                render_engine.initialize_additional_resources(&mut self.app.world);
                self.app.world.add_resource(render_engine);
            }
        }
    }
}

pub trait AppExtension {
    fn build(&self, app: &mut App);
}