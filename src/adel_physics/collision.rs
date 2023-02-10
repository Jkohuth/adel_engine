use crate::adel_ecs::{System, World};
use nalgebra::{Vector2, Vector3};
// BoxCollider2D Component
// Requires a location (offset from the parent/entity?)
// Requires extents
enum Direction {
    UP = 0,
    DOWN = 1,
    RIGHT = 2,
    LEFT = 3,
}
impl TryFrom<isize> for Direction {
    type Error = ();
    fn try_from(value: isize) -> Result<Direction, Self::Error> {
        match value {
            dir if dir == Direction::UP as isize => Ok(Direction::UP),
            dir if dir == Direction::DOWN as isize => Ok(Direction::DOWN),
            dir if dir == Direction::RIGHT as isize => Ok(Direction::RIGHT),
            dir if dir == Direction::LEFT as isize => Ok(Direction::LEFT),
            _ => Err(()),
        }
    }
}
impl Direction {
    #[allow(dead_code)]
    pub fn check_direction(direction: &Vector2<f32>) -> Direction {
        let compass: Vec<Vector2<f32>> = vec![
            Vector2::new(0.0, -1.0), // UP
            Vector2::new(0.0, 1.0),  // DOWN
            Vector2::new(1.0, 0.0),  // RIGHT
            Vector2::new(-1.0, 0.0), // LEFT
        ];
        let mut max: f32 = 0.0;
        let mut best_match: isize = -1;
        for i in compass.iter().enumerate() {
            let dot_product = Vector2::dot(&direction.normalize(), direction);
            if dot_product > max {
                max = dot_product;
                best_match = i.0 as isize;
            }
        }
        Direction::try_from(best_match).unwrap()
    }
}
pub trait Collider {
    fn center(&self) -> Vector3<f32>;
    fn check_direction(&self, _dir: Vector2<f32>) {}
    fn check_box_collision(&self, other: &BoxCollider2D) -> bool;
    fn name(&self) -> &'static str {
        "collider"
    }
}
pub struct BoxCollider2D {
    center: Vector3<f32>,
    _extents: Vector2<f32>,
}
impl Collider for BoxCollider2D {
    fn center(&self) -> Vector3<f32> {
        self.center
    }
    fn check_box_collision(&self, _other: &BoxCollider2D) -> bool {
        false
    }
}
impl System for dyn Collider {
    fn startup(&mut self, _world: &mut World) {}
    fn run(&mut self, _world: &mut World) {}
    fn shutdown(&mut self, _world: &mut World) {}
    fn name(&self) -> &'static str {
        self.name()
    }
}
// ColliderSystem2D
// Startup ()
// Run() -> Compare against
