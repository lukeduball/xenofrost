use cfg_if::cfg_if;
use winit::{event::{ElementState, Event, KeyEvent, WindowEvent}, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder};

use crate::core::{input_manager::InputManager, render_engine::RenderEngine};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

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
    let window = WindowBuilder::new().build(&event_loop).unwrap();

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

    let size = window.inner_size();
    let mut render_engine = RenderEngine::new(&window, size.width, size.height).await;
    let mut input_manager = InputManager::new();

    let _ = event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == render_engine.window().id() => {
            input_manager.process_input(event);
            match event {
                WindowEvent::CloseRequested | WindowEvent::KeyboardInput { 
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => control_flow.exit(),
                WindowEvent::Resized(physical_size) => {
                    render_engine.resize(physical_size.width, physical_size.height);
                },
                WindowEvent::RedrawRequested => {
                    //state.update();
                    if !render_engine.render_event() {
                        control_flow.exit();
                    }
                },
                _ => {}
            }
        },
        Event::AboutToWait => {
            render_engine.window().request_redraw();
        },
        _ => {}
    });
}