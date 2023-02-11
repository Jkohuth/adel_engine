use crate::adel_ecs::{System, World};
use crate::adel_input::InputConsumer;
use std::collections::HashSet;
use winit::event::VirtualKeyCode;
use winit::window::Window;

use crate::adel_camera::Camera;
use crate::adel_renderer::definitions::TransformComponent;
// This class will be a struct that contains the current input variables
// Which keys and which state shall be contained in this class
// Other Classes need to reference this class in order to update accordingly
// Starting point ECS to close window and exit game

// This Component is attached to entities who can react to keyboard inputs
// Initially this will be used for a single controlled object
pub struct KeyboardComponent;

pub struct KeyboardHandler {
    name: &'static str,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            name: "KeyboardHandler",
        }
    }
}

impl System for KeyboardHandler {
    fn startup(&mut self, world: &mut World) {
        let window = world.get_resource::<Window>().unwrap();
        let mut camera = world.get_resource_mut::<Camera>().unwrap();

        let mut transform_ref = world.borrow_component_mut::<TransformComponent>().unwrap();
        let input_ref = world.borrow_component::<KeyboardComponent>().unwrap();

        let dims = window.inner_size();
        let aspect_ratio = dims.width as f32 / dims.height as f32;
        camera.set_perspective_projection((50.0f32).to_radians(), aspect_ratio, 0.1, 10.0);

        for i in input_ref.iter().enumerate() {
            if let Some(_input_entity) = i.1 {
                if let Some(camera_transform) = &mut transform_ref[i.0] {
                    camera.set_view_target(
                        camera_transform.translation,
                        nalgebra::Vector3::<f32>::new(0.0, 0.0, 0.0),
                        Some(nalgebra::Vector3::<f32>::new(0.0, 0.0, 1.0)),
                    );
                }
            }
        }
    }

    fn run(&mut self, world: &mut World) {
        let input_consumer = world.get_resource::<InputConsumer>().unwrap();
        // No input, don't spend anymore time here
        if input_consumer.pressed.is_empty() {
            return;
        }

        let input_ref = world.borrow_component::<KeyboardComponent>().unwrap();
        let mut transform_ref = world.borrow_component_mut::<TransformComponent>().unwrap();
        let mut camera = world.get_resource_mut::<Camera>().unwrap();

        for i in input_ref.iter().enumerate() {
            // _input_entity is used to track that this entity at this position in the Component Array exists
            if let Some(_input_entity) = i.1 {
                if let Some(camera_transform) = &mut transform_ref[i.0] {
                    move_in_plane_xz(&input_consumer.pressed, world.get_dt(), camera_transform);
                    camera.set_view_target(
                        camera_transform.translation,
                        nalgebra::Vector3::<f32>::new(0.0, 0.0, 0.0),
                        Some(nalgebra::Vector3::<f32>::new(0.0, 0.0, 1.0)),
                    );
                }
            }
        }
    }

    fn shutdown(&mut self, _world: &mut World) {}
    fn name(&self) -> &str {
        self.name
    }
}

static LOOK_SPEED: f32 = 1.5;
static MOVE_SPEED: f32 = 3.0;

use nalgebra::Vector3;
#[allow(dead_code)]
fn move_2d_object(keys: &HashSet<VirtualKeyCode>, dt: f32, transform: &mut TransformComponent) {
    let mut move_dir = Vector3::default();
    if keys.contains(&VirtualKeyCode::D) {
        move_dir.x += 1.0;
    }
    if keys.contains(&VirtualKeyCode::A) {
        move_dir.x -= 1.0;
    }
    if keys.contains(&VirtualKeyCode::W) {
        move_dir.y += 1.0;
    }
    if keys.contains(&VirtualKeyCode::S) {
        move_dir.y -= 1.0;
    }

    if Vector3::dot(&move_dir, &move_dir) > f32::EPSILON {
        transform.translation += MOVE_SPEED * dt * move_dir.normalize();
    }
}

// TODO: Alter Camera Movement script for more control including rotation
fn move_in_plane_xz(
    keys: &HashSet<VirtualKeyCode>,
    dt: f32,
    camera_transform: &mut TransformComponent,
) {
    let mut rotate = Vector3::new(0.0, 0.0, 0.0);
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
    if Vector3::dot(&rotate, &rotate) > f32::EPSILON {
        camera_transform.rotation += LOOK_SPEED * dt * rotate.normalize();
    }

    // This is kinda dumb, look into making it more elegant
    camera_transform.rotation.x = camera_transform.rotation.x.clamp(-1.5, 1.5);
    camera_transform.rotation.y = camera_transform.rotation.y % (2.0 * std::f32::consts::PI);

    let yaw = camera_transform.rotation.y;
    let forward_dir = Vector3::new(yaw.sin(), 0.0, yaw.cos());
    let right_dir = Vector3::new(forward_dir.z, 0.0, -forward_dir.x);
    let up_dir = Vector3::new(0.0, -1.0, 0.0);
    let mut move_dir = Vector3::new(0.0, 0.0, 0.0);

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
    if Vector3::dot(&move_dir, &move_dir) > f32::EPSILON {
        camera_transform.translation += MOVE_SPEED * dt * move_dir.normalize();
    }
}
