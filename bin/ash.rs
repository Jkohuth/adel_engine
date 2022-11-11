
use adel::app::Application;
use adel::ecs::World;
use adel::renderer_ash::utility::structures::{TriangleComponent, Vertex2d};
use nalgebra::{Vector2, Vector3};


fn main() {

    simple_logger::SimpleLogger::new().env().init().unwrap();
    let mut world: World = World::new();
    let triangle_entity = world.new_entity();
    let triangle = vec![
        Vertex2d { position: Vector2::new(0.0, -1.0), color: Vector3::new(0.0, 1.0, 0.0)},
        Vertex2d { position: Vector2::new(0.5, 0.5), color: Vector3::new(1.0, 0.0, 0.0)},
        Vertex2d { position: Vector2::new(0.0, 0.5), color: Vector3::new(0.0, 0.0, 1.0)},

    ];
    let triangle_entity2 = world.new_entity();
    let triangle2 = vec![
        Vertex2d { position: Vector2::new(-0.5, -0.5), color: Vector3::new(0.0, 1.0, 0.0)},
        Vertex2d { position: Vector2::new(-0.75, -0.5), color: Vector3::new(1.0, 0.0, 0.0)},
        Vertex2d { position: Vector2::new(-0.2, 0.5), color: Vector3::new(0.0, 0.0, 1.0)},

    ];
    let triangle_component = TriangleComponent::new(triangle);
    world.add_component_to_entity(triangle_entity, triangle_component);
    let triangle_component2 = TriangleComponent::new(triangle2);
    world.add_component_to_entity(triangle_entity2, triangle_component2);
    let random_entity = world.new_entity();
    let app = Application::new(world);
    app.main_loop();
}