use serde::{Serialize, Deserialize};

use crate::{vector_alias::Coordinate, voxel_index_error::VoxelIndexError};
use std::collections::HashMap;

#[typetag::serde]
pub trait AdvancedVoxel {}

// #[derive(Serialize, Deserialize)]
// struct Container {
//     elements : HashSet<(u32, u32)>
// }

// #[typetag::serde]
// impl AdvancedVoxel for Container {}

#[derive(Serialize, Deserialize)]
struct Chunk2 {
    grid: [[[i16; 4]; 4]; 4],
    advanced_map: Vec<(i16, Box<dyn AdvancedVoxel>)>,
}

#[derive(Serialize, Deserialize)]
struct Chunk4 {
    grid: [[[Chunk2; 4]; 4]; 4],
    zero_point: Coordinate,
}

struct VoxelProperties {}

struct VoxelTypeDefinitions {
    map: HashMap<i16, VoxelProperties>,
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

fn to_internal_unchecked(
    coord: Coordinate,
    zero_coord: Coordinate,
    internal_step: i32,
) -> Coordinate {
    let relative_coord = coord - zero_coord;
    let internal_coord = relative_coord / internal_step;
    internal_coord
}

impl Chunk4 {
    fn get_properties<'a>(
        &self,
        definitions: &'a VoxelTypeDefinitions,
        coord: Coordinate,
    ) -> Result<&'a VoxelProperties, VoxelIndexError> {
        let internal4 = to_internal(coord, self.zero_point, 4, 4)
            .ok_or(VoxelIndexError { value: coord })?;

        let chunk2 = &self.grid[internal4.x as usize][internal4.y as usize][internal4.z as usize];

        let internal2 = to_internal(coord, self.zero_point + internal4 * 4, 4, 1)
            .expect("wrong subchunk aquired");

        let voxel = chunk2.grid[internal2.x as usize][internal2.y as usize][internal2.z as usize];

        definitions
            .map
            .get(&voxel)
            .ok_or(VoxelIndexError { value: coord })
    }
}
