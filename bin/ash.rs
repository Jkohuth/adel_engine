
use adel::app::Application;
use adel::ecs::World;
fn main() {

    simple_logger::SimpleLogger::new().env().init().unwrap();
    let world: World = World::new();
    let app = Application::new(world);
    app.main_loop();
}