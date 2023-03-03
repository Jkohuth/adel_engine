use adel::app::Application;
use adel::ecs::World;
use adel::input::KeyboardComponent;
use adel::renderer::definitions::{vec3_to_vec4, PointLightComponent};
use adel::renderer::utility::model::ModelComponent;
use adel::renderer::TransformComponent;
use nalgebra::{Translation, Vector3, Vector4};
use nalgebra_glm as glm;
use std::path::Path;
fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let mut world = World::new();
    let mut entity_vector = load_models(&mut world);

    load_point_lights(&mut world, &mut entity_vector);
    let app = Application::new(world);
    app.main_loop();
}
fn load_point_lights(world: &mut World, entity_vector: &mut Vec<usize>) {
    let light_colors: Vec<Vector4<f32>> = vec![
        Vector4::new(1.0, 0.1, 0.1, 1.0),
        Vector4::new(0.1, 0.1, 1.0, 1.0),
        Vector4::new(0.1, 1.0, 0.1, 1.0),
        Vector4::new(1.0, 1.0, 0.1, 1.0),
        Vector4::new(0.1, 1.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
    ];
    for i in light_colors.iter().enumerate() {
        let mut point_light_transform = TransformComponent::default();
        point_light_transform.scale.x = 0.2;
        let axis = nalgebra::Unit::new_normalize(Vector3::new(0.0, -1.0, 0.0));
        let rotation = nalgebra::Matrix4::<f32>::identity()
            * nalgebra::Rotation3::from_axis_angle(
                &axis,
                ((i.0 as f32) * 2.0 * std::f32::consts::PI) / light_colors.len() as f32,
            )
            .to_homogeneous();
        let translation = rotation * nalgebra::Vector4::new(-1.0, -1.0, -1.0, 1.0);
        point_light_transform.translation =
            Vector3::new(translation.x, translation.y, translation.z);
        //log::info!("Rotation {:?}, Translation {:?}", rotation, translation);

        let point_light = PointLightComponent::builder()
            .position(vec3_to_vec4(point_light_transform.translation))
            .color(*i.1)
            .build();
        let point_light_entity = world.new_entity();
        world.add_component_to_entity(point_light_entity, point_light);
        world.add_component_to_entity(point_light_entity, point_light_transform);
        entity_vector.push(point_light_entity);
    }
}
fn load_models(world: &mut World) -> Vec<usize> {
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
    let mut entity_vector = Vec::new();
    let entity0 = world.new_entity();
    let entity1 = world.new_entity();
    let floor_entity = world.new_entity();
    entity_vector.push(entity0);
    entity_vector.push(entity1);
    entity_vector.push(floor_entity);

    let camera_entity = world.new_entity();
    world.add_component_to_entity(entity0, model_build0);
    world.add_component_to_entity(entity0, model_transform0);
    world.add_component_to_entity(entity1, model_build1);
    world.add_component_to_entity(entity1, model_transform1);
    world.add_component_to_entity(floor_entity, floor);
    world.add_component_to_entity(floor_entity, floor_transform);

    world.add_component_to_entity(camera_entity, camera_transform);
    world.add_component_to_entity(camera_entity, keyboard_component);
    entity_vector
}
