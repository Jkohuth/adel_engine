use adel::app::Application;
use adel::ecs::World;
use adel::input::KeyboardComponent;
use adel::renderer::utility::model::ModelComponent;
use adel::renderer::TransformComponent;
use std::path::Path;
fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let mut world = World::new();
    let model_build = ModelComponent::builder().load_model(Path::new("resources/flat_vase.obj"));
    //.load_texture(Path::new("resources/viking_room.png"));
    let mut camera_transform = TransformComponent::default();
    camera_transform.translation = nalgebra::Vector3::<f32>::new(0.0, -0.25, -0.5);
    let mut cube_transform = TransformComponent::default();
    cube_transform.scale.y = 0.5;
    let keyboard_component = KeyboardComponent {};
    let entity = world.new_entity();
    let camera_entity = world.new_entity();
    world.add_component_to_entity(entity, model_build);
    world.add_component_to_entity(entity, cube_transform);
    world.add_component_to_entity(camera_entity, camera_transform);
    world.add_component_to_entity(camera_entity, keyboard_component);
    let app = Application::new(world);
    app.main_loop();
}
