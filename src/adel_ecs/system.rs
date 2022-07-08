use crate::adel_ecs::world::World;

pub trait System {
    fn run(&mut self, world: &mut World);
    fn name(&self) -> &str;
}