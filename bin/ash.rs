
use adel::app::Application;
use adel::ecs::World;
use adel::input::KeyboardComponent;
use adel::renderer_ash::definitions::{TransformComponent, TriangleComponent, Vertex2d};
use nalgebra::{Vector2, Vector3};

fn main() {

    simple_logger::SimpleLogger::new().env().init().unwrap();
    let mut world: World = World::new();
    let triangle_entity = world.new_entity();
    let triangle = vec![
        Vertex2d { position: Vector2::new(0.0, -1.0), color: Vector3::new(1.0, 0.0, 0.0)},
        Vertex2d { position: Vector2::new(0.5, 0.0), color: Vector3::new(1.0, 0.0, 0.0)},
        Vertex2d { position: Vector2::new(0.0, 0.0), color: Vector3::new(1.0, 0.0, 0.0)},

    ];
    let triangle3 = vec![
        Vertex2d { position: Vector2::new(0.0, -1.0), color: Vector3::new(1.0, 0.0, 0.0)},
        Vertex2d { position: Vector2::new(0.5, -1.0), color: Vector3::new(1.0, 0.0, 0.0)},
        Vertex2d { position: Vector2::new(0.5, 0.0), color: Vector3::new(1.0, 0.0, 0.0)},

    ];

    let transform = TransformComponent::new(
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 0.0)
    );

    let camera_controller_transform: TransformComponent = TransformComponent::new(
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(1.0, 1.0, 1.0),
        Vector3::new(0.0, 0.0, 0.0),
    );
    let triangle_entity2 = world.new_entity();
    let triangle2 = vec![
        Vertex2d { position: Vector2::new(-1.0, -1.0), color: Vector3::new(0.0, 1.0, 0.0)},
        Vertex2d { position: Vector2::new(1.0, 1.0), color: Vector3::new(0.0, 1.0, 0.0)},
        Vertex2d { position: Vector2::new(-1.0, 1.0), color: Vector3::new(0.0, 0.0, 1.0)},

    ];
    let transform2 = TransformComponent::new(
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 0.0)
    );

    let triangle_component = TriangleComponent::new(triangle);
    world.add_component_to_entity(triangle_entity, triangle_component);
    let triangle_component3 = TriangleComponent::new(triangle3);
    let triangle_entity3 = world.new_entity();
    world.add_component_to_entity(triangle_entity3, triangle_component3);
    world.add_component_to_entity(triangle_entity, transform);
    let triangle_component2 = TriangleComponent::new(triangle2);
    world.add_component_to_entity(triangle_entity2, triangle_component2);
    world.add_component_to_entity(triangle_entity2, transform2);
    let camera_entity = world.new_entity();
    world.add_component_to_entity(camera_entity, camera_controller_transform);
    //world.add_component_to_entity(camera_entity, KeyboardComponent);
    let random_entity = world.new_entity();
    let app = Application::new(world);
    app.main_loop();
}