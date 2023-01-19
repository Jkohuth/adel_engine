use adel::app::Application;
use adel::ecs::{World};
use adel::renderer_ash::utility::model::{ModelComponentBuilder, ModelComponent};
use adel::input::KeyboardHandler;
use adel::renderer_ash::TransformComponent;
use std::path::Path;
use adel::input::KeyboardComponent;
use nalgebra;
fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let mut world = World::new();
    let mut model_build = ModelComponentBuilder::new();
    model_build.load_model(Path::new("resources/colored_cube.obj"));
    let camera_transform = TransformComponent::default();
    let mut cube_transform = TransformComponent::default();
    cube_transform.translation.z += 5.0;
    let keyboard_component = KeyboardComponent{};
    //model_build.load_texture(Path::new("resources/viking_room.png"));
    let entity = world.new_entity();
    let camera_entity= world.new_entity();
    world.add_component_to_entity(entity, model_build);
    world.add_component_to_entity(entity, cube_transform);
    world.add_component_to_entity(camera_entity, camera_transform);
    world.add_component_to_entity(camera_entity, keyboard_component);
    let app = Application::new(world);
    app.main_loop();

}
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
