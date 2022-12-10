use std::{collections::HashMap};

pub trait AdvancedVoxel {}

struct Vector3i {
    x : i32,
    y : i32,
    z : i32,
}

impl std::ops::Add<Vector3i> for Vector3i {
    type Output = Vector3i;

    fn add(self, rhs: Vector3i) -> Self::Output {
        Vector3i { x : self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl std::ops::Sub<Vector3i> for Vector3i {
    type Output = Vector3i;

    fn sub(self, rhs: Vector3i) -> Self::Output {
        Vector3i { x : self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl std::ops::Mul<i32> for Vector3i {
    type Output = Vector3i;

    fn mul(self, rhs: i32) -> Self::Output {
        Vector3i { x : self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl std::ops::Div<i32> for Vector3i {
    type Output = Vector3i;

    fn div(self, rhs: i32) -> Self::Output {
        Vector3i { x : self.x / rhs, y: self.y / rhs, z: self.z / rhs }
    }
}

struct Chunk4 {
    grid : [i16; 4*4*4],
    advanced_map : HashMap<i16, Box<dyn AdvancedVoxel>>,
}

struct Chunk8 {
    grid : [Chunk4; 4*4*4],
    zero_point : Vector3i,
}

struct VoxelProperties {

}

struct VoxelTypeDefinitions
{
    map : HashMap<i16, VoxelProperties>,
}

struct VectorOutOfRangeError {
    value : Vector3i,
}

fn to_internal(coord : Vector3i, zero_coord : Vector3i, extend : i32, internal_step : i32) -> Option<Vector3i>
{
    let relative_coord = coord - zero_coord;
    let internal_coord = relative_coord / internal_step;

    if internal_coord.x < 0 { return None; }
    if internal_coord.x > (extend * extend) { return None; }
    if internal_coord.y < 0 { return None; }
    if internal_coord.y > (extend * extend) { return None; }
    if internal_coord.z < 0 { return None; }
    if internal_coord.z > (extend * extend) { return None; }
    Some(internal_coord)
}

impl Chunk8 {
    fn get_properties(&self, definitions : VoxelTypeDefinitions, coord : Vector3i) -> Result<&VoxelProperties, VectorOutOfRangeError>
    {
        let internal = to_internal(coord, self.zero_point, 4, 4)
                .ok_or(VectorOutOfRangeError {value : coord})?;

        let index = internal.x + internal.y * 4 + internal.z * 4 * 4;
        let chunk = self.grid[index as usize];

        let internal4 = to_internal(coord, self.zero_point + internal * 4, 4, 1)
                .expect("wrong subchunk aquired");
                
        let index4 = internal.x + internal.y * 4 + internal.z * 4 * 4;
        let voxel = chunk.grid[index4 as usize];

        definitions.map.get(&voxel).ok_or(VectorOutOfRangeError {value : coord})
    }
}
