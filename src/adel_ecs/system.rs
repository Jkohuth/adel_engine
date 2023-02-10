use crate::adel_ecs::world::World;

#[derive(PartialEq)]
pub enum RunStage {
    EarlyUpdate,
    Update,
    RedrawUpdate,
    LateUpdate,
}

pub trait System {
    fn startup(&mut self, world: &mut World);
    fn run(&mut self, world: &mut World);
    fn shutdown(&mut self, world: &mut World);
    fn name(&self) -> &str;

    fn get_run_stage(&self) -> RunStage {
        RunStage::Update
    }
}
