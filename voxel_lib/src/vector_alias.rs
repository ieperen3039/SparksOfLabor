use std::ops::Mul;

use nalgebra::{Point3, UnitQuaternion, UnitVector3, Vector3};
use serde::{Deserialize, Serialize};

pub type ICoordinate = Vector3<usize>;
pub type Coordinate = Vector3<i32>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Coordinate16(Vector3<i32>);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Coordinate64(Vector3<i32>);

pub type Vector3f = Vector3<f32>;
pub type Position = Point3<f32>;
pub type Direction = UnitVector3<f32>;
pub type Rotation = UnitQuaternion<f32>;

pub fn coordinate_containing_position(pos: &Position) -> Coordinate {
    Coordinate::new(pos.x as i32, pos.y as i32, pos.z as i32)
}

impl Coordinate16 {
    pub fn new(x: i32, y: i32, z: i32) -> Coordinate16 {
        Coordinate16(Vector3::new(x, y, z))
    }

    pub fn inner(&self) -> Vector3<i32> {
        self.0
    }

    pub fn add(self, x: i32, y: i32, z: i32) -> Self {
        Coordinate16(self.0 + Vector3::new(x, y, z))
    }

    pub fn containing_pos(pos : &Position) -> Coordinate16 {
        Self::new(
            (pos.x / 16.0) as i32,
            (pos.y / 16.0) as i32,
            (pos.z / 16.0) as i32,
        )
    }

    pub fn containing_coord(coord : &Coordinate) -> Coordinate16 {
        Self::new(coord.x / 16, coord.y / 16, coord.z / 16)
    }
}

impl Coordinate64 {
    pub fn new(x: i32, y: i32, z: i32) -> Coordinate64 {
        Coordinate64(Vector3::new(x, y, z))
    }

    pub fn inner(&self) -> Vector3<i32> {
        self.0
    }

    pub fn add(self, x: i32, y: i32, z: i32) -> Self {
        Coordinate64(self.0 + Vector3::new(x, y, z))
    }

    pub fn containing_position(pos : &Position) -> Coordinate64 {
        Self::new(
            (pos.x / 64.0) as i32,
            (pos.y / 64.0) as i32,
            (pos.z / 64.0) as i32,
        )
    }

    pub fn containing_coord(coord : &Coordinate) -> Coordinate64 {
        Self::new(coord.x / 64, coord.y / 64, coord.z / 64)
    }

    pub fn containing_coord16(coord : &Coordinate16) -> Coordinate64 {
        Self::new(coord.0.x / 16, coord.0.y / 16, coord.0.z / 16)
    }
}

impl From<Coordinate16> for Coordinate {
    fn from(coord: Coordinate16) -> Self {
        coord.0.mul(16)
    }
}

impl From<Coordinate64> for Coordinate {
    fn from(coord: Coordinate64) -> Self {
        coord.0.mul(64)
    }
}

impl From<Coordinate64> for Coordinate16 {
    fn from(coord: Coordinate64) -> Self {
        Coordinate16(coord.0.mul(16))
    }
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
