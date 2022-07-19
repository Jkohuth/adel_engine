use std::collections::HashSet;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use winit::event_loop::ControlFlow;
use crate::adel_input::InputConsumer;
use crate::adel_ecs::{System, World};
use crate::adel_renderer::TransformComponent;
use glam::{Vec3};
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

static LOOK_SPEED: f32 = 3.0;

fn move_in_plane_xz(keys: HashSet<VirtualKeyCode>, dt: f32, camera_transform: &mut TransformComponent) {

    let mut rotate = Vec3::new(0.0, 0.0, 0.0);
    // Look Right
    if keys.contains(&VirtualKeyCode::Right) {
        rotate.y += 1.0;
    }
    // Look Left
    if keys.contains(&VirtualKeyCode::Left) {
        rotate.y -= 1.0;
    }
    // Look Up
    if keys.contains(&VirtualKeyCode::Up) {
        rotate.x += 1.0;
    }
    // Look Down
    if keys.contains(&VirtualKeyCode::Down) {
        rotate.x -= 1.0;
    }
    if Vec3::dot(rotate, rotate) > f32::MIN {
        // I'm about to wrestle with some data type;
        camera_transform.rotation += LOOK_SPEED * dt * rotate.normalize();
        //let normalized_rotation = cgmath::InnerSpace::normalize(rotate);
        //let rad_norm_rotation = Vector3::<Rad<f32>>::new(Rad(normalized_rotation.x), Rad(normalized_rotation.y), Rad(normalized_rotation.z));
    }
}