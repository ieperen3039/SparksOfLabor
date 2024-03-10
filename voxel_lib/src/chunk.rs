use serde::{Deserialize, Serialize};

use crate::voxel::{Voxel, VoxelRef};
use crate::{
    block::{BaseVoxel, VoxelOrientation},
    block_types::BlockType,
    vector_alias::{Coordinate, ICoordinate},
    voxel_errors::VoxelIndexError,
    voxel_properties::{VoxelProperties, VoxelTypeDefinitions},
};

#[derive(Serialize, Deserialize)]
enum VoxelOrIndex {
    Direct(BaseVoxel),
    Mapped(u16),
}

#[derive(Serialize, Deserialize)]
pub struct Chunk64 {
    voxels: [[[Chunk16; 4]; 4]; 4],
    zero_coordinate: Coordinate,
}

#[derive(Serialize, Deserialize)]
struct BlockMapping {
    id: u16,
    element: Voxel,
}

#[derive(Serialize, Deserialize)]
struct Chunk16 {
    grid: Chunk16Grid,
    palette: Vec<BlockMapping>,
    zero_coordinate: Coordinate,
}

#[derive(Serialize, Deserialize)]
enum Chunk16Grid {
    // 2^16 = 65536 different element types, but there are only 4096 voxels per chunk.
    // we instead put all simple voxels directly in the grid, and only map the advanced voxels
    B32([[[VoxelOrIndex; 16]; 16]; 16]),
    // 2^8 = 256 different element types
    B8([[[u8; 16]; 16]; 16]),
    // 2^4 = 16 different element types
    B4([[[u8; 8]; 16]; 16]),
    // 2^2 = 4 different element types
    B2([[[u8; 4]; 16]; 16]),
}

impl VoxelRef<'_> {
    pub fn get_base(&self) -> BaseVoxel {
        match self {
            VoxelRef::Simple(&v) => v,
            VoxelRef::Advanced(v) => v.get_base_block(),
        }
    }
}

fn to_internal(
    coord: Coordinate,
    zero_coord: Coordinate,
    internal_step: i32,
    elements_in_grid: i32,
) -> Option<ICoordinate> {
    let relative_coord = coord - zero_coord;
    let internal_coord = relative_coord / internal_step;

    if internal_coord.x < 0 {
        return None;
    }
    if internal_coord.x > (elements_in_grid * elements_in_grid) {
        return None;
    }
    if internal_coord.y < 0 {
        return None;
    }
    if internal_coord.y > (elements_in_grid * elements_in_grid) {
        return None;
    }
    if internal_coord.z < 0 {
        return None;
    }
    if internal_coord.z > (elements_in_grid * elements_in_grid) {
        return None;
    }

    Some(ICoordinate::new(
        internal_coord.x as usize,
        internal_coord.y as usize,
        internal_coord.z as usize,
    ))
}

fn to_internal_unchecked(
    coord: Coordinate,
    zero_coord: Coordinate,
    internal_step: i32,
) -> ICoordinate {
    let relative_coord = coord - zero_coord;
    let internal_coord = relative_coord / internal_step;
    ICoordinate::new(
        internal_coord.x as usize,
        internal_coord.y as usize,
        internal_coord.z as usize,
    )
}

fn from_internal(coord: ICoordinate, zero_coord: Coordinate, internal_step: i32) -> Coordinate {
    zero_coord + (Coordinate::new(coord.x as i32, coord.y as i32, coord.z as i32) * internal_step)
}

impl Chunk64 {
    pub fn get_chunk16<'s>(&'s self, coord: Coordinate) -> Result<&'s Chunk16, VoxelIndexError> {
        let internal_coord = to_internal(coord, self.zero_coordinate, 16, 4)
            .ok_or(VoxelIndexError { coordinate: coord })?;

        Ok(self.get_chunk16_internal_unchecked(internal_coord))
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

    pub fn for_each<Action: FnMut(ICoordinate, VoxelRef)>(&self, mut action: Action) {
        // I regret nothing
        for x16 in 0..4usize {
            for y16 in 0..4usize {
                for z16 in 0..4usize {
                    let chunk16 = &self.voxels[x16][y16][z16];
                    let index_vector_16_base = ICoordinate::new(x16, y16, z16) * 16;
                    for x in 0..16usize {
                        for y in 0..16usize {
                            for z in 0..16usize {
                                let index_vector = ICoordinate::new(x, y, z);

                                let voxel = todo!();

                                let coord = index_vector_16_base + index_vector;
                                action(coord, voxel);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Chunk16 {
    pub fn get_properties<'a>(
        &self,
        definitions: &'a VoxelTypeDefinitions,
        coord: Coordinate,
    ) -> Result<&'a VoxelProperties, VoxelIndexError> {
        let voxel = self.get_voxel(coord)?;

        definitions
            .get_properties_of(voxel.get_base())
            .ok_or(VoxelIndexError { coordinate: coord })
    }

    pub fn get_voxel(&self, coord: Coordinate) -> Result<VoxelRef, VoxelIndexError> {
        let internal_coord = to_internal(coord, self.zero_coordinate, 16, 16)
            .ok_or(VoxelIndexError { coordinate: coord })?;

        Ok(self.get_voxel_internal_unchecked(internal_coord))
    }

    pub fn get_voxel_internal_unchecked(&self, coord: ICoordinate) -> VoxelRef {
        let index = match &self.grid {
            Chunk16Grid::B32(grid) => {
                let either = &grid[coord.x][coord.y][coord.z];
                match either {
                    VoxelOrIndex::Direct(block) => return VoxelRef::Simple(&block),
                    VoxelOrIndex::Mapped(idx) => idx.to_owned(),
                }
            },
            Chunk16Grid::B8(grid) => {
                grid[coord.x][coord.y][coord.z] as u16
            },
            Chunk16Grid::B4(grid) => todo!(),
            Chunk16Grid::B2(grid) => todo!(),
        };

        let search_result = self.palette.binary_search_by_key(&index, |m| m.id).expect("id in grid not found in palette");
        self.palette[search_result].element.as_voxel_ref()
    }
}