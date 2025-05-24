use std::collections::HashMap;

use winit::{event::{KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

use super::world::resource::Resource;

pub struct KeyState {
    is_down: bool,
    was_down: bool,
    was_pressed: bool,
    was_released: bool,
}

impl KeyState {
    fn new() -> KeyState {
        KeyState {
            is_down: false,
            was_down: false,
            was_pressed: false,
            was_released: false,
        }
    }

    /// Returns true is the key is currently pressed down, otherwise false
    pub fn get_is_down(&self) -> bool {
        self.is_down
    }

    /// Returns true if the key was pressed down for one iteration of the application, otherwise false
    #[allow(dead_code)]
    pub fn get_was_pressed(&self) -> bool {
        self.was_pressed
    }

    /// Returns true if the key was un-pressed for one iteration of the application, otherwise false
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
        input_manager.create_key_binding("atlas_toggle", KeyCode::Space);

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
                    key_state.is_down = state.is_pressed();
                }
            }
            _ => ()
        }
    }

    pub fn process_button_press_release_data(&mut self) {
        for state in self.key_state.values_mut() {
            if state.is_down && !state.was_down {
                state.was_pressed = true;
            } else {
                state.was_pressed = false;
            }

            if !state.is_down && state.was_down {
                state.was_released = true;
            } else {
                state.was_released = false;
            }
            state.was_down = state.is_down;
        }
    }


}