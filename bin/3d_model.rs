use adel::app::Application;
use adel::ecs::World;
use adel::input::KeyboardComponent;
use adel::renderer::utility::model::ModelComponent;
use adel::renderer::TransformComponent;
use nalgebra::Vector3;
use std::path::Path;
fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let mut world = World::new();
    let floor = ModelComponent::builder().load_model(Path::new("resources/quad.obj"));
    let mut floor_transform = TransformComponent::new(
        Vector3::new(0.0, 0.5, 0.0),
        Vector3::new(3.0, 1.0, 3.0),
        Vector3::default(),
    );
    let mut camera_transform = TransformComponent::default();
    camera_transform.translation = nalgebra::Vector3::<f32>::new(0.0, -0.25, -2.5);

    let model_build0 = ModelComponent::builder().load_model(Path::new("resources/flat_vase.obj"));
    let mut model_transform0 = TransformComponent::new(
        Vector3::new(0.5, 0.5, 0.0),
        Vector3::new(3.0, 1.5, 3.0),
        Vector3::default(),
    );

    let model_build1 = ModelComponent::builder().load_model(Path::new("resources/flat_vase.obj"));
    let mut model_transform1 = TransformComponent::new(
        Vector3::new(-0.5, 0.5, 0.0),
        Vector3::new(3.0, 1.5, 3.0),
        Vector3::default(),
    );
    model_transform1.translation = nalgebra::Vector3::<f32>::new(-0.5, 0.5, 0.0);
    let keyboard_component = KeyboardComponent {};
    let entity = world.new_entity();
    let entity2 = world.new_entity();
    let floor_entity = world.new_entity();

    let camera_entity = world.new_entity();
    world.add_component_to_entity(entity, model_build0);
    world.add_component_to_entity(entity, model_transform0);
    world.add_component_to_entity(entity2, model_build1);
    world.add_component_to_entity(entity2, model_transform1);
    world.add_component_to_entity(floor_entity, floor);
    world.add_component_to_entity(floor_entity, floor_transform);

    world.add_component_to_entity(camera_entity, camera_transform);
    world.add_component_to_entity(camera_entity, keyboard_component);
    let app = Application::new(world);
    app.main_loop();
}
