use serde::{Deserialize, Serialize};

use crate::{
    vector_alias::{Coordinate, ICoordinate},
    voxel::ByteVoxel,
    voxel_errors::VoxelIndexError,
    voxel_properties::{VoxelProperties, VoxelTypeDefinitions},
};

#[typetag::serde]
pub trait AdvancedVoxel {
    fn get_base_block(&self) -> ByteVoxel;
}

// could be a typedef
#[derive(Serialize, Deserialize)]
pub struct SimpleVoxel(ByteVoxel);

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

impl VoxelRef<'_> {
    pub fn get_base(&self) -> ByteVoxel {
        match self {
            VoxelRef::Simple(SimpleVoxel(v)) => *v,
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

    Some(ICoordinate::new(internal_coord.x as usize, internal_coord.y as usize, internal_coord.z as usize))
}

fn to_internal_unchecked(
    coord: Coordinate,
    zero_coord: Coordinate,
    internal_step: i32,
) -> ICoordinate {
    let relative_coord = coord - zero_coord;
    let internal_coord = relative_coord / internal_step;
    ICoordinate::new(internal_coord.x as usize, internal_coord.y as usize, internal_coord.z as usize)
}

fn from_internal(
    coord: ICoordinate,
    zero_coord: Coordinate,
    internal_step: i32,
) -> Coordinate {
    zero_coord + (Coordinate::new(coord.x as i32, coord.y as i32, coord.z as i32) * internal_step)
}

impl<T> Grid444<T> {
    pub fn get<'s>(&'s self, coord: ICoordinate) -> &'s T {
        &self.grid[coord.x][coord.y][coord.z]
    }
}

impl Chunk64 {
    pub fn get_chunk16<'s>(&'s self, coord: Coordinate) -> Result<&'s Chunk16, VoxelIndexError> {
        let internal_coord = to_internal(coord, self.zero_coordinate, 16, 4)
            .ok_or(VoxelIndexError { coordinate: coord })?;

        Ok(self.voxels.get(internal_coord))
    }

    pub fn get_coordinate_from_index(&self, block_index: ICoordinate) -> Coordinate {
        let zero_coord = self.zero_coordinate;
        zero_coord + Coordinate::new(block_index.x as i32, block_index.y as i32, block_index.z as i32)
    }

    pub fn for_each<Action: FnMut(ICoordinate, VoxelRef)>(&self, mut action: Action) {
        // I regret nothing
        for x16 in 0..4usize {
            for y16 in 0..4usize {
                for z16 in 0..4usize {
                    let chunk16 = &self.voxels.grid[x16][y16][z16];
                    let index_16 = ICoordinate::new(x16, y16, z16) * 16;
                    for x4 in 0..4usize {
                        for y4 in 0..4usize {
                            for z4 in 0..4usize {
                                let chunk4_coord = ICoordinate::new(x4, y4, z4);
                                let index_4 = chunk4_coord * 4;
                                for x in 0..4usize {
                                    for y in 0..4usize {
                                        for z in 0..4usize {
                                            let chunk1_coord = ICoordinate::new(x, y, z);

                                            let byte_voxel = match &chunk16.voxels {
                                                Chunk16Grid::Uniform(voxel) => VoxelRef::Simple(voxel),
                                                Chunk16Grid::Simple(voxels) => {
                                                    let grid = voxels.get(chunk4_coord);
                                                    VoxelRef::Simple(grid.get(chunk1_coord))
                                                },
                                                Chunk16Grid::Detailed(voxels) => {
                                                    let chunk4 = voxels.get(chunk4_coord);
                                                    chunk4.get_voxel_internal(chunk1_coord)
                                                },
                                            };

                                            let coord = index_16 + index_4 + chunk1_coord;
                                            action(coord, byte_voxel);
                                        }
                                    }
                                }
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
        let chunk4_coord = to_internal(coord, self.zero_coordinate, 4, 4)
            .ok_or(VoxelIndexError { coordinate: coord })?;

        let chunk4_zero_coord = from_internal(chunk4_coord, self.zero_coordinate, 4);
        let internal1 = to_internal_unchecked(coord, chunk4_zero_coord, 1);

        match &self.voxels {
            Chunk16Grid::Uniform(voxel) => Ok(VoxelRef::Simple(voxel)),
            Chunk16Grid::Simple(voxels) => {
                let grid = voxels.get(chunk4_coord);
                Ok(VoxelRef::Simple(grid.get(internal1)))
            },
            Chunk16Grid::Detailed(voxels) => {
                let chunk4 = voxels.get(chunk4_coord);
                Ok(chunk4.get_voxel_internal(internal1))
            },
        }
    }
}

impl Chunk4 {
    pub fn get_voxel_internal(&self, coord: ICoordinate) -> VoxelRef {
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

    pub fn get_block_internal(&self, coord: ICoordinate) -> ByteVoxel {
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
