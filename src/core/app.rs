use std::{sync::Arc, time::Instant};

use winit::{application::ApplicationHandler, event::{ElementState, KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy}, keyboard::{KeyCode, PhysicalKey}, window::WindowId};

use super::{input_manager::InputManager, render_engine::RenderEngine};

//Micro seconds needed to be used because the integer duration of ms could be zero causing no updates for many frames
const FRAMES_PER_UPDATE_MICRO_SECONDS: u128 = 1600;

#[allow(dead_code)]
pub struct App<WorldData, RenderData> {
    app_name: &'static str,
    window: Option<Arc<winit::window::Window>>,
    startup_hook: fn(&mut WorldData, &mut RenderData, &mut InputManager, &RenderEngine),
    resize_hook: fn(&mut WorldData, &mut RenderData, &RenderEngine),
    update_hook: fn(&mut WorldData, &InputManager),
    prepare_hook: fn(&mut WorldData, &mut RenderData, &RenderEngine),
    render_hook: fn(&RenderData, &RenderEngine) -> Result<(), wgpu::SurfaceError>,
    input_manager: InputManager,
    render_engine: Option<RenderEngine>,
    world_data: WorldData,
    render_data: RenderData,

    is_startup: bool,
    previous_update_time: Instant,
    lag: u128,
}

impl<WorldData, RenderData> App<WorldData, RenderData> {
    pub fn new(
        app_name: &'static str, 
        world_data: WorldData, 
        render_data: RenderData, 
        startup_hook: fn(&mut WorldData, &mut RenderData, &mut InputManager, &RenderEngine),
        resize_hook: fn(&mut WorldData, &mut RenderData, &RenderEngine), 
        update_hook: fn(&mut WorldData, &InputManager),
        prepare_hook: fn(&mut WorldData, &mut RenderData, &RenderEngine), 
        render_hook: fn(&RenderData, &RenderEngine) -> Result<(), wgpu::SurfaceError>
    ) -> Self {
        Self {
            app_name,
            window: None,
            startup_hook,
            resize_hook,
            update_hook,
            prepare_hook,
            render_hook,
            input_manager: InputManager::new(),
            render_engine: None,
            world_data,
            render_data,
            is_startup: true,
            previous_update_time: Instant::now(),
            lag: 0,
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::with_user_event().build().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut app_runner = AppRunner::new(self, &event_loop);
        _ = event_loop.run_app(&mut app_runner);
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.render_engine.as_mut().unwrap().resize(new_width, new_height);
        }
    }
}

//Used by wasm32 build so warning was turned off
#[allow(dead_code)]
enum EngineEvent {
    CreateGraphicsEvent(RenderEngine)
}

//Event loop proxy is used by wasm32 so warning was turned off
#[allow(dead_code)]
struct AppRunner<'a, WorldData, RenderData> {
    app: &'a mut App<WorldData, RenderData>,
    event_loop_proxy: Option<EventLoopProxy<EngineEvent>>
}

impl<'a, WorldData, RenderData> AppRunner<'a, WorldData, RenderData> {
    fn new(app: &'a mut App<WorldData, RenderData>, event_loop: &EventLoop<EngineEvent>) -> Self {
        Self {
            app,
            event_loop_proxy: Some(event_loop.create_proxy())
        }
    }
}

impl<'a, WorldData, RenderData> ApplicationHandler<EngineEvent> for AppRunner<'a, WorldData, RenderData> {
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

        self.app.window = Some(Arc::clone(&arc_window));

        #[cfg(target_arch="wasm32")]
        {
            let event_loop_proxy = self.event_loop_proxy.take();
            let render_engine_fut = RenderEngine::new(Arc::clone(&arc_window), size.width, size.height, self.app.render_hook);
            wasm_bindgen_futures::spawn_local(async move {
                let render_engine = render_engine_fut.await;
                assert!(event_loop_proxy.unwrap().send_event(EngineEvent::CreateGraphicsEvent(render_engine)).is_ok());
            });
        }

        #[cfg(not(target_arch="wasm32"))]
        {
            let render_engine = pollster::block_on(RenderEngine::new(arc_window, size.width, size.height));
            self.app.render_engine = Some(render_engine);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if let None = self.app.render_engine {
            //Wait until the graphics have been created
            return;
        }
        let scale_factor = self.app.window.as_ref().unwrap().scale_factor();

        if self.app.is_startup {
            (self.app.startup_hook)(&mut self.app.world_data, &mut self.app.render_data, &mut self.app.input_manager, &self.app.render_engine.as_ref().unwrap());
            self.app.is_startup = false;
        }

        self.app.input_manager.process_input(&event, scale_factor);

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
                    (self.app.resize_hook)(&mut self.app.world_data, &mut self.app.render_data, &self.app.render_engine.as_ref().unwrap());
                }
            },
            WindowEvent::RedrawRequested => {
                let elapsed_time = self.app.previous_update_time.elapsed();
                self.app.previous_update_time = Instant::now();
                self.app.lag += elapsed_time.as_micros();

                while self.app.lag > FRAMES_PER_UPDATE_MICRO_SECONDS {
                    self.app.input_manager.process_button_press_release_data();
                    (self.app.update_hook)(&mut self.app.world_data, &self.app.input_manager);
                    self.app.lag -= FRAMES_PER_UPDATE_MICRO_SECONDS;
                    (self.app.prepare_hook)(&mut self.app.world_data, &mut self.app.render_data, &self.app.render_engine.as_mut().unwrap());
                }

                let app_render_result = (self.app.render_hook)(&self.app.render_data, &self.app.render_engine.as_ref().unwrap());
                if !self.app.render_engine.as_mut().unwrap().successful_render(app_render_result) {
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
                self.app.render_engine = Some(render_engine);
            }
        }
    }
}