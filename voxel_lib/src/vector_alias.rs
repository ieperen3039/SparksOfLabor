use std::ops::Mul;

use nalgebra::{Point3, UnitQuaternion, UnitVector3, Vector3};
use serde::{Deserialize, Serialize};

pub type ICoordinate = Vector3<usize>;
pub type Coordinate = Vector3<i32>;
pub type Coordinate16 = Vector3<i32>;
pub type Coordinate64 = Vector3<i32>;

pub type Vector3f = Vector3<f32>;
pub type Position = Point3<f32>;
pub type Direction = UnitVector3<f32>;
pub type Rotation = UnitQuaternion<f32>;

pub fn coordinate16_to_absolute(coord: Coordinate16) -> Coordinate {
    coord.mul(16)
}

pub fn coordinate64_to_absolute(coord: Coordinate64) -> Coordinate {
    coord.mul(64)
}

pub fn coordinate64_to_coordinate16(coord: Coordinate64) -> Coordinate {
    coord.mul(16)
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum AxisDirection {
    PosX,
    PosY,
    PosZ,
    NegX,
    NegY,
    NegZ,
}

impl AxisDirection {
    pub fn get_unit(&self) -> Direction {
        match self {
            AxisDirection::PosX => Direction::new_unchecked(nalgebra::vector![1.0, 0.0, 0.0]),
            AxisDirection::PosY => Direction::new_unchecked(nalgebra::vector![0.0, 1.0, 0.0]),
            AxisDirection::PosZ => Direction::new_unchecked(nalgebra::vector![0.0, 0.0, 1.0]),
            AxisDirection::NegX => Direction::new_unchecked(nalgebra::vector![-1.0, 0.0, 0.0]),
            AxisDirection::NegY => Direction::new_unchecked(nalgebra::vector![0.0, -1.0, 0.0]),
            AxisDirection::NegZ => Direction::new_unchecked(nalgebra::vector![0.0, 0.0, -1.0]),
        }
    }
}
