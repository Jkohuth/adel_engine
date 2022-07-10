use std::collections::HashSet;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use winit::event_loop::ControlFlow;
use crate::adel_input::InputConsumer;
use crate::adel_ecs::{System, World};
use crate::adel_renderer::TransformComponent;
use std::time::Instant;
// This class will be a struct that contains the current input variables
// Which keys and which state shall be contained in this class
// Other Classes need to reference this class in order to update accordingly
// Starting point ESC to close window and exit game

// This Component is attached to entities who can react to keyboard inputs
// Initially this will be used for a single controlled object
pub struct KeyboardComponent;

pub struct KeyboardHandler {
    name: &'static str,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            name: "KeyboardComponent",
        }
    }

}

impl System for KeyboardHandler {
    fn run(&mut self, world: &mut World) {
        let input_consumer = world.get_resource::<InputConsumer>().unwrap();
        
        // No input, don't spend anymore time here
        if input_consumer.pressed.is_empty() {
            return;
        }

        let input_ref = world.borrow_component::<KeyboardComponent>().unwrap();
        let mut transform_ref = world.borrow_component_mut::<TransformComponent>().unwrap();

        for i in input_ref.iter().enumerate() {
            // _input_entity is used to track that this entity at this position in the Component Array exists 
            if let Some(_input_entity) = i.1 {
                let camera_transform = transform_ref.get_mut(i.0).unwrap();
                log::info!("Transform and Input component {:?}", camera_transform);
            }
        }
    }
    fn name(&self) -> &str {
        self.name
    }
}