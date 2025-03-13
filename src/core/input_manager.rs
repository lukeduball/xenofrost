use std::collections::HashMap;

use winit::{event::{KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

use super::world::resource::Resource;

pub struct KeyState {
    is_down: bool,
    was_pressed: bool,
    was_released: bool,
}

impl KeyState {
    fn new() -> KeyState {
        KeyState {
            is_down: false,
            was_pressed: false,
            was_released: false,
        }
    }

    pub fn get_is_down(&self) -> bool {
        self.is_down
    }

    #[allow(dead_code)]
    pub fn get_was_pressed(&self) -> bool {
        self.was_pressed
    }

    #[allow(dead_code)]
    pub fn get_was_released(&self) -> bool {
        self.was_released
    }
}

#[derive(Resource)]
pub struct InputManager {
    key_binding: HashMap<KeyCode, &'static str>,
    key_state: HashMap<&'static str, KeyState>
}

impl InputManager {
    pub fn new() -> InputManager {
        let mut input_manager = Self {
            key_binding: HashMap::new(),
            key_state: HashMap::new()
        };

        input_manager.create_key_binding("left", KeyCode::KeyA);
        input_manager.create_key_binding("right", KeyCode::KeyD);
        input_manager.create_key_binding("up", KeyCode::KeyW);
        input_manager.create_key_binding("down", KeyCode::KeyS);

        input_manager
    }

    fn create_key_binding(&mut self, key_identifier: &'static str, key_code: KeyCode) {
        self.key_binding.insert(key_code, key_identifier);
        self.key_state.insert(key_identifier, KeyState::new());
    }

    pub fn get_key_state(&self, key_identifier: &str) -> Option<&KeyState> {
        self.key_state.get(key_identifier)
    }

    pub fn process_input(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event: 
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let key_identifier_option = self.key_binding.get(keycode);
                if let Some(key_identifer) = key_identifier_option {
                    let key_state = self.key_state.get_mut(key_identifer).unwrap();
                    if key_state.is_down {
                        if !state.is_pressed() {
                            key_state.was_released = true;
                        }
                    } else {
                        if state.is_pressed() {
                            key_state.was_pressed = true;
                        }
                    }
                    key_state.is_down = state.is_pressed();
                }
            }
            _ => ()
        }
    }


}