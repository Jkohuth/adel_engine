use adel::app::Application;
use adel::ecs::{World};
use adel::renderer::{ModelBuilder, ModelComponent, TransformComponent, Vertex};
use adel::input::KeyboardComponent;
use glam::{Vec3};
fn main() {
        simple_logger::SimpleLogger::new().env().init().unwrap();
    let obj_file = String::from("/home/jakob/projects/blender_models/obj_files/cube.obj");
    /*//let obj_file = String::from("/home/jakob/projects/blender_models/obj_files/scythe.obj");
    std::process::exit(0); */
    let transform: TransformComponent = TransformComponent::new(
        Vec3::new(0.0, 0.0, 2.5),
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
    world.add_component_to_entity(cube_entity, ModelBuilder::load_model(obj_file.as_str()));
    world.add_component_to_entity(cube_entity, transform);
    let camera_entity = world.new_entity();
    world.add_component_to_entity(camera_entity, camera_controller_transform);
    world.add_component_to_entity(camera_entity, KeyboardComponent);
    let app = Application::new(world);
    app.main_loop();
}
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
