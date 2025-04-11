use crate::chunk16::Chunk16;
use crate::vector_alias::{Coordinate, Coordinate16, Coordinate64, ICoordinate};
use crate::voxel::VoxelRef;
use crate::voxel_errors::VoxelIndexError;
use serde::{Deserialize, Serialize};
use std::array;

use minecraft_protocol::{
    components::{blocks as mc_blocks, chunk as mc_chunk},
    ids::blocks as mc_ids,
};

#[derive(Serialize, Deserialize)]
pub struct Chunk64 {
    voxels: [[[Chunk16; 4]; 4]; 4],
    zero_coordinate: Coordinate,
}

impl Chunk64 {
    pub fn new(coordinate: Coordinate64, fill_value: mc_ids::Block) -> Self {
        let coord16 = Coordinate16::from(coordinate);

        let voxels = array::from_fn(|y| {
            array::from_fn(|z| {
                array::from_fn(|x| {
                    Chunk16::new(coord16.add(x as i32, y as i32, z as i32), fill_value)
                })
            })
        });

        Chunk64 {
            voxels,
            zero_coordinate: Coordinate::from(coordinate),
        }
    }

    fn get_chunk16_internal_unchecked(&self, internal_coord: ICoordinate) -> &Chunk16 {
        &self.voxels[internal_coord.x][internal_coord.y][internal_coord.z]
    }

    pub fn get_coordinate_from_index(&self, block_index: ICoordinate) -> Coordinate {
        let zero_coord = self.zero_coordinate;
        zero_coord
            + Coordinate::new(
                block_index.x as i32,
                block_index.y as i32,
                block_index.z as i32,
            )
    }
}
