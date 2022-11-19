use crate::adel_ecs::world::World;

pub trait System {
    fn startup(&mut self, world: &mut World);
    fn run(&mut self, world: &mut World);
    fn shutdown(&mut self, world: &mut World);
    fn name(&self) -> &str;
}