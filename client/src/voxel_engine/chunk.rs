use crate::vector_alias::Coordinate;
use std::collections::HashMap;

pub trait AdvancedVoxel {}

struct Chunk2 {
    grid: [i16; 4 * 4 * 4],
    advanced_map: HashMap<i16, Box<dyn AdvancedVoxel>>,
}

struct Chunk4 {
    grid: [Chunk2; 4 * 4 * 4],
    zero_point: Coordinate,
}

struct VoxelProperties {}

struct VoxelTypeDefinitions {
    map: HashMap<i16, VoxelProperties>,
}

struct VectorOutOfRangeError {
    value: Coordinate,
}

fn to_internal(
    coord: Coordinate,
    zero_coord: Coordinate,
    extend: i32,
    internal_step: i32,
) -> Option<Coordinate> {
    let internal_coord = to_internal_unchecked(coord, zero_coord, internal_step);

    if internal_coord.x < 0 {
        return None;
    }
    if internal_coord.x > (extend * extend) {
        return None;
    }
    if internal_coord.y < 0 {
        return None;
    }
    if internal_coord.y > (extend * extend) {
        return None;
    }
    if internal_coord.z < 0 {
        return None;
    }
    if internal_coord.z > (extend * extend) {
        return None;
    }

    Some(internal_coord)
}

fn to_internal_unchecked(coord: Coordinate, zero_coord: Coordinate, internal_step: i32) -> Coordinate {
    let relative_coord = coord - zero_coord;
    let internal_coord = relative_coord / internal_step;
    internal_coord
}

impl Chunk4 {
    fn get_properties<'a>(
        &self,
        definitions: &'a VoxelTypeDefinitions,
        coord: Coordinate,
    ) -> Result<&'a VoxelProperties, VectorOutOfRangeError> {
        let internal4 = to_internal(coord, self.zero_point, 4, 4)
            .ok_or(VectorOutOfRangeError { value: coord })?;

        let index4 = internal4.x + internal4.y * 4 + internal4.z * 4 * 4;
        let chunk2 = &self.grid[index4 as usize];

        let internal2 = to_internal(coord, self.zero_point + internal4 * 4, 4, 1)
            .expect("wrong subchunk aquired");

        let index2 = internal2.x + internal2.y * 4 + internal2.z * 4 * 4;
        let voxel = chunk2.grid[index2 as usize];

        definitions
            .map
            .get(&voxel)
            .ok_or(VectorOutOfRangeError { value: coord })
    }
}
