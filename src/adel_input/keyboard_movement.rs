use std::collections::HashSet;
use winit::{
        event::{VirtualKeyCode},
};
use crate::adel_input::InputConsumer;
use crate::adel_ecs::{System, World};

use crate::adel_camera::Camera;
use crate::adel_renderer::definitions::{TransformComponent};
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
    fn startup(&mut self, _world: &mut World) {
        /* Window object should be a world resource so it can be gotten anywhere
        let input_ref = world.borrow_component::<KeyboardComponent>().unwrap();
        let mut transform_ref = world.borrow_component_mut::<TransformComponent>().unwrap();
        for i in input_ref.iter().enumerate() {
            // _input_entity is used to track that this entity at this position in the Component Array exists
            if let Some(_input_entity) = i.1 {
                if let Some(camera_transform) = &mut transform_ref[i.0] {
                    //log::info!("Post move camera_transform {:?} dt {:?}", &camera_transform, world.get_dt());
                    let mut camera = world.get_resource_mut::<Camera>().unwrap();
                    camera.set_view_yxz(camera_transform.translation, camera_transform.rotation);
                    camera.set_perspective_projection(fovy, aspect, near, far)
                    //camera.set_orthographic_projection(-1.0, 1.0, 1.0, -1.0, -1.0, 10.0);
                }
            }
        } */
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

                    //move_2d_object(&input_consumer.pressed, world.get_dt(), transform);
                    //camera.set_orthographic_projection_pos(1.0, 1.0, 10.0);
                    //log::info!("Camera Info Position: {:?}\nProjection: {:?}", camera_transform, camera.get_projection());
                    //log::info!("Input consumed Tranform {:?}", &transform);
                    //log::info!("Inside the move script camera_transform {:?} dt {:?}", &camera_transform, world.get_dt());
                    move_in_plane_xz(&input_consumer.pressed, world.get_dt(), camera_transform);
                    //log::info!("Post move camera_transform {:?} dt {:?}", &camera_transform, world.get_dt());
                    camera.set_view_yxz(camera_transform.translation, camera_transform.rotation);

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

use nalgebra::{Vector3};
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

fn move_in_plane_xz(keys: &HashSet<VirtualKeyCode>, dt: f32, camera_transform: &mut TransformComponent) {

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
    camera_transform.rotation.y = camera_transform.rotation.y % (2.0*std::f32::consts::PI);

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