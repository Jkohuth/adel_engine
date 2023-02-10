use adel::app::Application;
use adel::ecs::World;
use adel::input::KeyboardComponent;
use adel::renderer::utility::model::ModelComponent;
use adel::renderer::TransformComponent;
use std::path::Path;
fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let mut world = World::new();
    let model_build = ModelComponent::builder()
        .load_model(Path::new("resources/viking_room.obj"))
        .load_texture(Path::new("resources/viking_room.png"));
    let camera_transform = TransformComponent::default();
    let cube_transform = TransformComponent::default();
    //cube_transform.translation.z += 5.0;
    //cube_transform.rotation.x += 180.0;
    let keyboard_component = KeyboardComponent {};
    //model_build.load_texture(Path::new("resources/viking_room.png"));
    let entity = world.new_entity();
    let camera_entity = world.new_entity();
    world.add_component_to_entity(entity, model_build);
    world.add_component_to_entity(entity, cube_transform);
    world.add_component_to_entity(camera_entity, camera_transform);
    world.add_component_to_entity(camera_entity, keyboard_component);
    let app = Application::new(world);
    app.main_loop();
}