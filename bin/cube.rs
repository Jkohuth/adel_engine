use adel::app::Application;
use adel::ecs::{World};
//use adel::renderer::{ModelBuilder, ModelComponent, TransformComponent, Vertex};
use adel::input::KeyboardComponent;
//use glam::{Vec3};

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    /*let builder = create_cube_model(Vec3::new(0.0, 0.0, 0.0));
    let builder2 = create_cube_model(Vec3::new(0.0, -2.0, 0.0));
    let transform: TransformComponent = TransformComponent::new(
        Vec3::new(0.0, 0.0, 2.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(0.0, 0.0, 0.0),
    );
    let transform2: TransformComponent = TransformComponent::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(0.0, 0.0, 0.0),
    );
    let camera_controller_transform: TransformComponent = TransformComponent::new(
        Vec3::new(-1.0, 2.0, -2.0),
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(0.0, 0.0, 0.0),
    );
    let mut world: World = World::new();
    let cube_entity = world.new_entity();
    world.add_component_to_entity(cube_entity, ModelComponent::new(builder));
    world.add_component_to_entity(cube_entity, transform);
    let cube_entity2 = world.new_entity();
    world.add_component_to_entity(cube_entity2, ModelComponent::new(builder2));
    world.add_component_to_entity(cube_entity2, transform2);
    let camera_entity = world.new_entity();
    world.add_component_to_entity(camera_entity, camera_controller_transform);
    world.add_component_to_entity(camera_entity, KeyboardComponent);
    let app = Application::new(world);
    app.main_loop();*/
}
/*
fn create_cube_model(offset: Vec3) -> ModelBuilder {
    let mut verticies = vec![
        // left face (white)
        Vertex { position: [-0.5, -0.5, -0.5], color: [0.9, 0.9, 0.9]},
        Vertex { position: [-0.5,  0.5,  0.5], color: [0.9, 0.9, 0.9]},
        Vertex { position: [-0.5, -0.5,  0.5], color: [0.9, 0.9, 0.9]},
        Vertex { position: [-0.5,  0.5, -0.5], color: [0.9, 0.9, 0.9]},
        // right face (yellow)
        Vertex { position: [0.5, -0.5, -0.5], color: [0.8, 0.8, 0.1]},
        Vertex { position: [0.5,  0.5,  0.5], color: [0.8, 0.8, 0.1]},
        Vertex { position: [0.5, -0.5,  0.5], color: [0.8, 0.8, 0.1]},
        Vertex { position: [0.5,  0.5, -0.5], color: [0.8, 0.8, 0.1]},
        // top face (orange, remember y axis points down)
        Vertex { position: [-0.5, -0.5, -0.5], color: [0.9, 0.6, 0.1]},
        Vertex { position: [ 0.5, -0.5,  0.5], color: [0.9, 0.6, 0.1]},
        Vertex { position: [-0.5, -0.5,  0.5], color: [0.9, 0.6, 0.1]},
        Vertex { position: [ 0.5, -0.5, -0.5], color: [0.9, 0.6, 0.1]},
        // bottom face (red)
        Vertex { position: [-0.5, 0.5, -0.5], color: [0.8, 0.1, 0.1]},
        Vertex { position: [ 0.5, 0.5,  0.5], color: [0.8, 0.1, 0.1]},
        Vertex { position: [-0.5, 0.5,  0.5], color: [0.8, 0.1, 0.1]},
        Vertex { position: [ 0.5, 0.5, -0.5], color: [0.8, 0.1, 0.1]},
        // nose face (blue)
        Vertex { position: [-0.5, -0.5, 0.5], color: [0.1, 0.1, 0.8]},
        Vertex { position: [ 0.5,  0.5, 0.5], color: [0.1, 0.1, 0.8]},
        Vertex { position: [-0.5,  0.5, 0.5], color: [0.1, 0.1, 0.8]},
        Vertex { position: [ 0.5, -0.5, 0.5], color: [0.1, 0.1, 0.8]},
        // tail face (green)
        Vertex { position: [-0.5, -0.5, -0.5], color: [0.1, 0.8, 0.1]},
        Vertex { position: [ 0.5,  0.5, -0.5], color: [0.1, 0.8, 0.1]},
        Vertex { position: [-0.5,  0.5, -0.5], color: [0.1, 0.8, 0.1]},
        Vertex { position: [ 0.5, -0.5, -0.5], color: [0.1, 0.8, 0.1]},
    ];
    for i in verticies.iter_mut() {
        i.position[0] += offset.x;
        i.position[1] += offset.y;
        i.position[2] += offset.z;
    }
    let indicies = vec![0,  1,  2,  0,  3,  1,  4,  5,  6,  4,  7,  5,  8,  9,  10, 8,  11, 9,
                          12, 13, 14, 12, 15, 13, 16, 17, 18, 16, 19, 17, 20, 21, 22, 20, 23, 21];
    ModelBuilder::new(verticies, indicies)
}
 Will remove, usually keep old data around for a commit or two
// temporary helper function, creates a 1x1x1 cube centered at offset
fn create_cube_model_bak(offset: Vec3)  -> ModelComponent{
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
}*/