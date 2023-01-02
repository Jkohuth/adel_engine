
use adel::app::Application;
use adel::ecs::World;
use adel::renderer_ash::definitions::{VertexIndexComponent, Vertex, Vertex2d};
use nalgebra::{Vector2, Vector3};
use adel::renderer_ash::definitions::{TransformComponent, Transform2dComponent};
use adel::input::KeyboardComponent;
use nalgebra::Matrix4;
fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let mut world = World::new();

    let vertices: Vec<Vertex> = vec![
        Vertex { position: Vector3::new(-0.5, -0.5, 0.0), color: Vector3::new(1.0, 0.0, 0.0), tex_coord: Vector2::new(1.0, 0.0) },
        Vertex { position: Vector3::new( 0.5, -0.5, 0.0), color: Vector3::new(0.0, 1.0, 0.0), tex_coord: Vector2::new(0.0, 0.0) },
        Vertex { position: Vector3::new( 0.5,  0.5, 0.0), color: Vector3::new(0.0, 0.0, 1.0), tex_coord: Vector2::new(0.0, 1.0) },
        Vertex { position: Vector3::new(-0.5,  0.5, 0.0), color: Vector3::new(1.0, 1.0, 1.0), tex_coord: Vector2::new(1.0, 1.0) },
    ];
    let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];

    let vi_component = VertexIndexComponent {
        vertices,
        indices: indices.clone()
    };
    let mut transform = TransformComponent::default();
    transform.translation.x -= 0.5;
    let camera_transform = TransformComponent::default();

    let keyboard = KeyboardComponent {};
    let entity = world.new_entity();
    world.add_component_to_entity(entity, vi_component);
    world.add_component_to_entity(entity, transform);
    world.add_component_to_entity(entity, keyboard);
    let camera_entity = world.new_entity();
    world.add_component_to_entity(camera_entity, camera_transform);
    let app = Application::new(world);
    app.main_loop();
}