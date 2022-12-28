
use adel::app::Application;
use adel::ecs::World;
use adel::renderer_ash::definitions::{Vertex2dIndexComponent, Vertex2d};
use nalgebra::{Vector2, Vector3};
use adel::renderer_ash::definitions::{TransformComponent, Transform2dComponent};
use adel::input::KeyboardComponent;

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let mut world = World::new();

    let vertices: Vec<Vertex2d> = vec![
        Vertex2d { position: Vector2::new(-0.5, -0.5), color: Vector3::new(1.0, 0.0, 0.0) },
        Vertex2d { position: Vector2::new(0.5, -0.5), color: Vector3::new(0.0, 0.0, 1.0) },
        Vertex2d { position: Vector2::new(0.5, 0.5), color: Vector3::new(0.0, 1.0, 0.0) },
        Vertex2d { position: Vector2::new(-0.5, 0.5), color: Vector3::new(1.0, 0.0, 0.0) },
    ];
    let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];

    let vertices2: Vec<Vertex2d> = vec![
        Vertex2d { position: Vector2::new(-0.5, -0.5), color: Vector3::new(0.0, 0.0, 1.0)},
        Vertex2d { position: Vector2::new(0.5, -0.5), color: Vector3::new(0.0, 0.0, 1.0)},
        Vertex2d { position: Vector2::new(0.5, 0.5), color: Vector3::new(0.0, 0.0, 1.0)},
    ];
    let indices2: Vec<u16> = vec![0, 1, 2];
    let vi_component2 = Vertex2dIndexComponent {
        vertices: vertices2,
        indices: indices2
    };
    let vi_component = Vertex2dIndexComponent {
        vertices,
        indices
    };
    let transform = Transform2dComponent::default();
    let keyboard = KeyboardComponent {};
    let entity = world.new_entity();
    world.add_component_to_entity(entity, vi_component2);
    world.add_component_to_entity(entity, transform);
    world.add_component_to_entity(entity, keyboard);
    let app = Application::new(world);
    app.main_loop();
}
