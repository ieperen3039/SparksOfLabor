use serde::{Deserialize, Serialize};

use crate::{
    block_types::Block,
    vector_alias::Coordinate,
    voxel_index_error::VoxelIndexError,
    voxel_properties::{VoxelProperties, VoxelTypeDefinitions},
};

#[typetag::serde]
pub trait AdvancedVoxel {
    fn get_base_block(&self) -> Block;
}

// #[derive(Serialize, Deserialize)]
// struct Container {
//     elements : u32
// }

// #[typetag::serde]
// impl AdvancedVoxel for Container {
//     fn get_base_block(&self) -> Block {
//         return Block::Container;
//     }
// }

// could be a typedef
#[derive(Serialize, Deserialize)]
pub struct SimpleVoxel(Block);

#[derive(Serialize, Deserialize)]
pub enum Voxel {
    // bitfield of type + variant + rotation
    Simple(SimpleVoxel),
    // reference to heap-allocated voxel definition
    Advanced(Box<dyn AdvancedVoxel>),
}

pub enum VoxelRef<'a> {
    // bitfield of type + variant + rotation
    Simple(&'a SimpleVoxel),
    // reference to heap-allocated voxel definition
    Advanced(&'a dyn AdvancedVoxel),
}

#[derive(Serialize, Deserialize)]
struct Grid444<T> {
    grid: [[[T; 4]; 4]; 4],
}

// chunk of 264 bytes
#[derive(Serialize, Deserialize)]
enum Chunk4Grid {
    // a chunk containing just one type of block (air, slate)
    Uniform(SimpleVoxel),
    // a chunk containing only simple voxels
    // 256 bytes
    Simple(Grid444<SimpleVoxel>),
    // any other chunk containing any combination of voxel types
    // box of 1024 bytes
    Detailed(Box<Grid444<Voxel>>),
}

#[derive(Serialize, Deserialize)]
pub struct Chunk4 {
    voxels: Chunk4Grid,
}

#[derive(Serialize, Deserialize)]
enum Chunk16Grid {
    // a chunk16 containing just one type of block (air, slate)
    Uniform(SimpleVoxel),
    // a chunk16 consisting only out of simple chunks
    // box of 16_384 bytes / 16 KB
    Simple(Box<Grid444<Grid444<SimpleVoxel>>>),
    // any combination of chunks
    // box of 16_896 bytes / 16.5 KB
    Detailed(Box<Grid444<Chunk4>>),
}

#[derive(Serialize, Deserialize)]
pub struct Chunk16 {
    voxels: Chunk16Grid,
    zero_coordinate: Coordinate,
}

#[derive(Serialize, Deserialize)]
pub struct Chunk64 {
    // 2.048 bytes / 2 KB
    voxels: Grid444<Chunk16>,
    zero_coordinate: Coordinate,
}

fn to_internal(
    coord: Coordinate,
    zero_coord: Coordinate,
    internal_step: i32,
    elements_in_grid: i32,
) -> Option<Coordinate> {
    let internal_coord = to_internal_unchecked(coord, zero_coord, internal_step);

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

impl<T> Grid444<T> {
    fn get<'s>(&'s self, coord: Coordinate) -> &'s T {
        &self.grid[coord.x as usize][coord.y as usize][coord.z as usize]
    }
}

impl Chunk64 {
    fn get_chunk16<'s>(&'s self, coord: Coordinate) -> Result<&'s Chunk16, VoxelIndexError> {
        let internal_coord = to_internal(coord, self.zero_coordinate, 16, 4)
            .ok_or(VoxelIndexError { value: coord })?;

        Ok(self.voxels.get(internal_coord))
    }
}

impl Chunk16 {
    fn get_properties<'a>(
        &self,
        definitions: &'a VoxelTypeDefinitions,
        coord: Coordinate,
    ) -> Result<&'a VoxelProperties, VoxelIndexError> {
        let voxel = self.get_block_type_absolute(coord)?;

        definitions
            .get_properties_of(voxel)
            .ok_or(VoxelIndexError { value: coord })
    }

    fn get_block_type_absolute(&self, coord: Coordinate) -> Result<Block, VoxelIndexError> {
        let internal4 = to_internal_unchecked(coord, self.zero_coordinate, 4);

        match &self.voxels {
            Chunk16Grid::Uniform(SimpleVoxel(voxel_type)) => Ok(*voxel_type),
            Chunk16Grid::Simple(voxels) => {
                let grid = voxels.get(internal4);
                let internal2 =
                    to_internal_unchecked(coord, self.zero_coordinate + internal4 * 4, 1);
                let SimpleVoxel(voxel_type) = grid.get(internal2);
                Ok(*voxel_type)
            },
            Chunk16Grid::Detailed(voxels) => {
                let chunk4 = voxels.get(internal4);
                let coord = to_internal_unchecked(coord, self.zero_coordinate + internal4 * 4, 1);

                Ok(chunk4.get_block_relative(coord))
            },
        }
    }
}

impl Chunk4 {
    fn get_voxel(&self, coord: Coordinate) -> VoxelRef {
        match &self.voxels {
            Chunk4Grid::Uniform(voxel) => VoxelRef::Simple(voxel),
            Chunk4Grid::Simple(voxels) => {
                let voxel = voxels.get(coord);
                VoxelRef::Simple(voxel)
            },
            Chunk4Grid::Detailed(voxels) => {
                let voxel_impl = voxels.get(coord);
                match voxel_impl {
                    Voxel::Simple(voxel) => VoxelRef::Simple(voxel),
                    Voxel::Advanced(voxel_box) => VoxelRef::Advanced(voxel_box.as_ref()),
                }
            },
        }
    }

    fn get_block_relative(&self, coord: Coordinate) -> Block {
        match &self.voxels {
            Chunk4Grid::Uniform(SimpleVoxel(voxel_type)) => *voxel_type,
            Chunk4Grid::Simple(voxels) => {
                let SimpleVoxel(voxel_type) = voxels.get(coord);
                *voxel_type
            },
            Chunk4Grid::Detailed(voxels) => {
                let voxel_impl = voxels.get(coord);
                match voxel_impl {
                    Voxel::Simple(SimpleVoxel(voxel_type)) => *voxel_type,
                    Voxel::Advanced(advanced_voxel) => advanced_voxel.get_base_block(),
                }
            },
        }
    }
}
