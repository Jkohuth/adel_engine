use adel::app::Application;
use adel::ecs::{World};
use adel::renderer::{ModelComponent, TransformComponent, Vertex};
use adel::input::KeyboardComponent;
use cgmath::{Vector3, Rad};

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    // TODO: Debug why the cube is showing graphical errors
    let model = create_cube_model(Vector3::<f32>::new(0.0, 0.0, 0.0)); 
    let transform: TransformComponent = TransformComponent::new(
        Vector3::<f32>::new(0.0, 0.0, 2.5),
        Vector3::<f32>::new(0.5, 0.5, 0.5),
        Vector3::<Rad<f32>>::new(Rad(0.0), Rad(0.0), Rad(0.0)),
    );
    let camera_controller_transform: TransformComponent = TransformComponent::new(
        Vector3::<f32>::new(-1.0, 2.0, -2.0),
        Vector3::<f32>::new(1.0, 1.0, 1.0),
        Vector3::<Rad<f32>>::new(Rad(0.0), Rad(0.0), Rad(0.0)),
    );
    let mut world: World = World::new();
    let cube_entity = world.new_entity();
    world.add_component_to_entity(cube_entity, model);
    world.add_component_to_entity(cube_entity, transform);
    let camera_entity = world.new_entity();
    world.add_component_to_entity(camera_entity, camera_controller_transform);
    world.add_component_to_entity(camera_entity, KeyboardComponent);
    let app = Application::new(world);
    app.main_loop();
}

// temporary helper function, creates a 1x1x1 cube centered at offset
fn create_cube_model(offset: Vector3::<f32>) -> ModelComponent {
    let mut verticies = vec![
        // left face (white)
        Vertex { position: [-0.5, -0.5, -0.5], color: [0.9, 0.9, 0.9]},
        Vertex { position: [-0.5,  0.5,  0.5], color: [0.9, 0.9, 0.9]},
        Vertex { position: [-0.5, -0.5,  0.5], color: [0.9, 0.9, 0.9]},
        Vertex { position: [-0.5, -0.5, -0.5], color: [0.9, 0.9, 0.9]},
        Vertex { position: [-0.5,  0.5, -0.5], color: [0.9, 0.9, 0.9]},
        Vertex { position: [-0.5,  0.5,  0.5], color: [0.9, 0.9, 0.9]},
  
        // right face (yellow)
        Vertex { position: [0.5, -0.5, -0.5], color: [0.8, 0.8, 0.1]},
        Vertex { position: [0.5,  0.5,  0.5], color: [0.8, 0.8, 0.1]},
        Vertex { position: [0.5, -0.5,  0.5], color: [0.8, 0.8, 0.1]},
        Vertex { position: [0.5, -0.5, -0.5], color: [0.8, 0.8, 0.1]},
        Vertex { position: [0.5,  0.5, -0.5], color: [0.8, 0.8, 0.1]},
        Vertex { position: [0.5,  0.5,  0.5], color: [0.8, 0.8, 0.1]},
  
        // top face (orange, remember y axis points down)
        Vertex { position: [-0.5, -0.5, -0.5], color: [0.9, 0.6, 0.1]},
        Vertex { position: [ 0.5, -0.5,  0.5], color: [0.9, 0.6, 0.1]},
        Vertex { position: [-0.5, -0.5,  0.5], color: [0.9, 0.6, 0.1]},
        Vertex { position: [-0.5, -0.5, -0.5], color: [0.9, 0.6, 0.1]},
        Vertex { position: [ 0.5, -0.5, -0.5], color: [0.9, 0.6, 0.1]},
        Vertex { position: [ 0.5, -0.5,  0.5], color: [0.9, 0.6, 0.1]},

        // bottom face (red)
        Vertex { position: [-0.5, 0.5, -0.5], color: [0.8, 0.1, 0.1]},
        Vertex { position: [ 0.5, 0.5,  0.5], color: [0.8, 0.1, 0.1]},
        Vertex { position: [-0.5, 0.5,  0.5], color: [0.8, 0.1, 0.1]},
        Vertex { position: [-0.5, 0.5, -0.5], color: [0.8, 0.1, 0.1]},
        Vertex { position: [ 0.5, 0.5, -0.5], color: [0.8, 0.1, 0.1]},
        Vertex { position: [ 0.5, 0.5,  0.5], color: [0.8, 0.1, 0.1]},
  
        // nose face (blue)
        Vertex { position: [-0.5, -0.5, 0.5], color: [0.1, 0.1, 0.8]},
        Vertex { position: [ 0.5,  0.5, 0.5], color: [0.1, 0.1, 0.8]},
        Vertex { position: [-0.5,  0.5, 0.5], color: [0.1, 0.1, 0.8]},
        Vertex { position: [-0.5, -0.5, 0.5], color: [0.1, 0.1, 0.8]},
        Vertex { position: [ 0.5, -0.5, 0.5], color: [0.1, 0.1, 0.8]},
        Vertex { position: [ 0.5,  0.5, 0.5], color: [0.1, 0.1, 0.8]},
  
        // tail face (green)
        Vertex { position: [-0.5, -0.5, -0.5], color:[0.1, 0.8, 0.1]},
        Vertex { position: [ 0.5,  0.5, -0.5], color:[0.1, 0.8, 0.1]},
        Vertex { position: [-0.5,  0.5, -0.5], color:[0.1, 0.8, 0.1]},
        Vertex { position: [-0.5, -0.5, -0.5], color:[0.1, 0.8, 0.1]},
        Vertex { position: [ 0.5, -0.5, -0.5], color:[0.1, 0.8, 0.1]},
        Vertex { position: [ 0.5,  0.5, -0.5], color:[0.1, 0.8, 0.1]},
    ];
    
    for i in verticies.iter_mut() {
        i.position[0] += offset.x;
        i.position[1] += offset.y;
        i.position[2] += offset.z;
    }
    ModelComponent::new(verticies)
}