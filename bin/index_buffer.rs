
use adel::app::Application;
use adel::ecs::World;
use adel::renderer_ash::definitions::{VertexIndexComponent, Vertex2d};
use nalgebra::{Vector2, Vector3};

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

    let vi_component = VertexIndexComponent {
        vertices,
        indices
    };
    let entity = world.new_entity();
    world.add_component_to_entity(entity, vi_component);
    let app = Application::new(world);
    app.main_loop();
}
