use std::collections::HashSet;
use std::cell::Ref;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use winit::event_loop::ControlFlow;
use crate::adel_input::InputConsumer;
use crate::adel_ecs::{System, World};
use crate::adel_renderer::TransformComponent;
use glam::{Vec3};
use crate::adel_camera::Camera;
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
    fn startup(&mut self, world: &mut World) {}

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
                if let Some(camera_transform) = &mut transform_ref[i.0] {
                    //log::info!("Inside the move script camera_transform {:?} dt {:?}", &camera_transform, world.get_dt());
                    move_in_plane_xz(&input_consumer.pressed, world.get_dt(), camera_transform);
                    //log::info!("Post move camera_transform {:?} dt {:?}", &camera_transform, world.get_dt());
                    let mut camera = world.get_resource_mut::<Camera>().unwrap();
                    camera.set_view_yxz(camera_transform.translation, camera_transform.rotation);

                }
            }
        }
    }
    fn name(&self) -> &str {
        self.name
    }
}

pub fn print_type_of<T>(_: &T) {
    log::info!("T is of Type {:?}", std::any::type_name::<T>());
}
static LOOK_SPEED: f32 = 1.5;
static MOVE_SPEED: f32 = 3.0;

fn move_in_plane_xz(keys: &HashSet<VirtualKeyCode>, dt: f32, camera_transform: &mut TransformComponent) {

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
    if Vec3::dot(rotate, rotate) > f32::EPSILON {
        camera_transform.rotation += LOOK_SPEED * dt * rotate.normalize();
    }

    camera_transform.rotation.x.clamp(-1.5, 1.5);
    camera_transform.rotation.y = camera_transform.rotation.y % (2.0*std::f32::consts::PI);

    let yaw = camera_transform.rotation.y;
    let forward_dir = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let right_dir = Vec3::new(forward_dir.z, 0.0, -forward_dir.x);
    let up_dir = Vec3::new(0.0, -1.0, 0.0);
    let mut move_dir = Vec3::new(0.0, 0.0, 0.0);

    // Move forward
    if keys.contains(&VirtualKeyCode::W) {
        move_dir += forward_dir;
    }
    // Move Back
    if keys.contains(&VirtualKeyCode::S) {
        move_dir -= forward_dir;
    }
    // Move Right
    if keys.contains(&VirtualKeyCode::D) {
        move_dir += right_dir;
    }
    // Move Left
    if keys.contains(&VirtualKeyCode::A) {
        move_dir -= right_dir;
    }
    // Move Up
    if keys.contains(&VirtualKeyCode::E) {
        move_dir += up_dir;
    }
    // Move Down
    if keys.contains(&VirtualKeyCode::Q) {
        move_dir -= up_dir;
    }
    if Vec3::dot(move_dir, move_dir) > f32::EPSILON {
        camera_transform.translation += MOVE_SPEED * dt * move_dir.normalize();
    }
}