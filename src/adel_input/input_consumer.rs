use std::collections::HashSet;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

#[derive(Debug, Clone)]
pub struct InputConsumer {
    pub pressed: HashSet<VirtualKeyCode>,
}

impl InputConsumer {
    pub fn keyboard_input_system(&mut self, keyboard_input: &KeyboardInput) {
        let key_code = keyboard_input.virtual_keycode.unwrap();
        match keyboard_input.state {
            ElementState::Pressed => { self.pressed.insert(key_code.clone()); },
            ElementState::Released => {
                if self.pressed.contains(&key_code) {
                    self.pressed.remove(&key_code);
                }
            },
        };
    }
}