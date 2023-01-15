use adel::app::Application;
use adel::ecs::{World};
use adel::renderer_ash::utility::model::{ModelComponentBuilder, ModelComponent};
use std::path::Path;
use adel::input::KeyboardComponent;
fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let mut world = World::new();
    let mut model_build = ModelComponentBuilder::new();
    model_build.load_model(Path::new("resources/viking_room.obj"));
    model_build.load_texture(Path::new("resources/viking_room.png"));
    let entity = world.new_entity();
    world.add_component_to_entity(entity, model_build);
    let app = Application::new(world);
    app.main_loop();

}
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
