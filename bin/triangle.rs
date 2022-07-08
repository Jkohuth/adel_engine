use adel::ecs::World;
use adel::app::Application;
use adel::renderer::{Vertex2d, TriangleComponent, Transform2dComponent};

use cgmath::Vector2;

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let verticies = vec![   Vertex2d { position: [-0.5, -0.5], color: [1.0, 0.0, 0.0], },
                            Vertex2d { position: [0.5,  -0.5], color: [1.0, 0.0, 0.0], },
                            Vertex2d { position: [-0.5,  0.5], color: [1.0, 0.0, 0.0], } ];

    let _verticies2 = vec![   Vertex2d { position: [-1.0, -1.0], color: [0.0, 0.0, 1.0], },
                             Vertex2d { position: [-0.5, -0.5], color: [0.0, 0.0, 1.0], },
                             Vertex2d { position: [-1.0,  0.0], color: [0.0, 0.0, 1.0], } ];
    

    

    let mut world = World::new();
    let triangle_entity = world.new_entity();
    world.add_component_to_entity(triangle_entity, TriangleComponent::new(verticies));
    world.add_component_to_entity(triangle_entity, Transform2dComponent::default());

    let transform: Transform2dComponent = Transform2dComponent::new(
            Vector2::new(0.25, 0.5), // Translation
            Vector2::new(1.5, 0.5),  // Scale
            std::f32::consts::FRAC_PI_2, // Rotation
        );
    world.add_component_to_entity(triangle_entity, transform);

    //let triangle2_entity = world.new_entity();
    //world.add_component_to_entity(triangle2_entity, TriangleComponent::new(verticies2));
    let app = Application::new(world);
    app.main_loop();
}
