use std::collections::HashSet;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use winit::event_loop::ControlFlow;
// This class will be a struct that contains the current input variables
// Which keys and which state shall be contained in this class
// Other Classes need to reference this class in order to update accordingly
// Starting point ESC to close window and exit game

#[derive(Debug, Clone)]
pub struct Input {
    pub pressed: HashSet<VirtualKeyCode>,
}

impl Input {
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

    pub fn keyboard_read_system(&self, control_flow: &mut ControlFlow) {
        if self.pressed.contains(&VirtualKeyCode::Escape) {
            *control_flow = ControlFlow::Exit;
        }
        else if !self.pressed.is_empty() {
            for i in self.pressed.iter() {
                println!("{:?}", i);
            }
        }
    }
}