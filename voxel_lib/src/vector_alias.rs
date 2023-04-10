use nalgebra::{Vector3, Point3, UnitVector3};

pub type Coordinate = Vector3<i32>;
pub type Position = Point3<f32>;
pub type Direction = UnitVector3<f32>;

pub const UNIT_X : Direction = Direction::new_unchecked(nalgebra::vector![1.0, 0.0, 0.0]);
pub const UNIT_Y : Direction = Direction::new_unchecked(nalgebra::vector![0.0, 1.0, 0.0]);
pub const UNIT_Z : Direction = Direction::new_unchecked(nalgebra::vector![0.0, 0.0, 1.0]);
pub const VEC_ZERO : Position = Position::new(0.0, 0.0, 0.0);