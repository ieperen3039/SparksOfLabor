use serde::{Deserialize, Serialize};

use crate::{vector_alias::Coordinate, voxel_index_error::VoxelIndexError, block_types::BlockType, voxel_properties::{VoxelTypeDefinitions, VoxelProperties}};

#[typetag::serde]
pub trait AdvancedVoxel {
    fn get_block_id(&self) -> BlockType;
}

// #[derive(Serialize, Deserialize)]
// struct Container {
//     elements : u32
// }

// #[typetag::serde]
// impl AdvancedVoxel for Container {
//     fn get_block_id(&self) -> u32 {
//         return 0;
//     }
// }

// could be a typedef
#[derive(Serialize, Deserialize)]
pub struct SimpleVoxel(BlockType);

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

#[derive(Serialize, Deserialize)]
enum Chunk4Grid {
    // a chunk containing just one type of block (air, slate)
    Uniform(SimpleVoxel),
    // a chunk containing only simple voxels
    Simple(Grid444<SimpleVoxel>),
    // any other chunk containing any combination of voxel types
    Detailed(Box<Grid444<Voxel>>),
}

#[derive(Serialize, Deserialize)]
struct Chunk4 {
    voxels: Chunk4Grid,
}

#[derive(Serialize, Deserialize)]
enum Chunk16Grid {
    // a chunk16 containing just one type of block (air, slate)
    Uniform(SimpleVoxel),
    // a chunk16 consisting only out of simple chunks
    Simple(Grid444<Grid444<SimpleVoxel>>),
    // any combination of chunks
    Detailed(Grid444<Chunk4>),
}

#[derive(Serialize, Deserialize)]
pub struct Chunk16 {
    voxels: Chunk16Grid,
    zero_coordinate: Coordinate,
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

impl Chunk16 {
    fn get_properties<'a>(
        &self,
        definitions: &'a VoxelTypeDefinitions,
        coord: Coordinate,
    ) -> Result<&'a VoxelProperties, VoxelIndexError> {
        let voxel = self.get_block_type_absolute(coord)?;

        definitions.get_properties_of(voxel)
            .ok_or(VoxelIndexError { value: coord })
    }

    fn get_block_type_absolute(
        &self,
        coord: Coordinate,
    ) -> Result<BlockType, VoxelIndexError> {
        let internal4 = to_internal(coord, self.zero_coordinate, 4, 4)
            .ok_or(VoxelIndexError { value: coord })?;

        match &self.voxels {
            Chunk16Grid::Uniform(SimpleVoxel(voxel_type)) => Ok(*voxel_type),
            Chunk16Grid::Simple(voxels) => {
                let grid =
                    &voxels.grid[internal4.x as usize][internal4.y as usize][internal4.z as usize];
                let internal2 =
                    to_internal_unchecked(coord, self.zero_coordinate + internal4 * 4, 1);
                let SimpleVoxel(voxel_type) =
                    grid.grid[internal2.x as usize][internal2.y as usize][internal2.z as usize];
                Ok(voxel_type)
            },
            Chunk16Grid::Detailed(voxels) => {
                let chunk4 =
                    &voxels.grid[internal4.x as usize][internal4.y as usize][internal4.z as usize];
                let coord = to_internal_unchecked(coord, self.zero_coordinate + internal4 * 4, 1);

                Ok(chunk4.get_block_type_relative(coord))
            },
        }
    }
}

impl Chunk4 {
    fn get_voxel(
        &self,
        coord: Coordinate,
    ) -> VoxelRef {
        match &self.voxels {
            Chunk4Grid::Uniform(voxel) => VoxelRef::Simple(voxel),
            Chunk4Grid::Simple(voxels) => {
                let voxel = &voxels.grid[coord.x as usize][coord.y as usize][coord.z as usize];
                VoxelRef::Simple(voxel)
            },
            Chunk4Grid::Detailed(voxels) => {
                let voxel_impl = &voxels.grid[coord.x as usize][coord.y as usize][coord.z as usize];
                match voxel_impl {
                    Voxel::Simple(voxel) => VoxelRef::Simple(voxel),
                    Voxel::Advanced(voxel_box) => VoxelRef::Advanced(voxel_box.as_ref()),
                }
            },
        }
    }

    fn get_block_type_relative(
        &self,
        coord: Coordinate,
    ) -> BlockType {
        match &self.voxels {
            Chunk4Grid::Uniform(SimpleVoxel(voxel_type)) => *voxel_type,
            Chunk4Grid::Simple(voxels) => {
                let SimpleVoxel(voxel_type) =
                    voxels.grid[coord.x as usize][coord.y as usize][coord.z as usize];
                voxel_type
            },
            Chunk4Grid::Detailed(voxels) => {
                let voxel_impl = &voxels.grid[coord.x as usize][coord.y as usize][coord.z as usize];
                match voxel_impl {
                    Voxel::Simple(SimpleVoxel(voxel_type)) => *voxel_type,
                    Voxel::Advanced(advanced_voxel) => advanced_voxel.get_block_id(),
                }
            },
        }
    }
}
