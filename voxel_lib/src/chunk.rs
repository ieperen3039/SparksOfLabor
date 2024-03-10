use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::voxel::{self, Voxel, VoxelRef};
use crate::{
    block::BaseVoxel,
    vector_alias::{Coordinate, ICoordinate},
    voxel_errors::VoxelIndexError,
    voxel_properties::{VoxelProperties, VoxelTypeDefinitions},
};

#[derive(Serialize, Deserialize)]
enum ChunkB32Entry {
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
    // 2^16 = 65536 different element types, and there are only 4096 voxels per chunk.
    id: u16,
    num_elements: u16,
    element: Voxel,
}

#[derive(Serialize, Deserialize)]
pub struct Chunk16 {
    grid: Chunk16Grid,
    palette: Vec<BlockMapping>,
    zero_coordinate: Coordinate,
}

#[derive(Serialize, Deserialize)]
enum Chunk16Grid {
    // 2^16 = 65536 different element types, but there are only 4096 voxels per chunk.
    // we instead put all simple voxels directly in the grid for 32 bits per voxel, and only map the advanced voxels
    B32(Box<[[[ChunkB32Entry; 16]; 16]; 16]>),
    // 2^8 = 256 different element types
    B8(Box<[[[u8; 16]; 16]; 16]>),
    // 2^4 = 16 different element types
    B4(Box<[[[u8; 8]; 16]; 16]>),
    // 2^2 = 4 different element types
    B2(Box<[[[u8; 4]; 16]; 16]>),
}

impl VoxelRef<'_> {
    pub fn get_base(&self) -> BaseVoxel {
        match self {
            VoxelRef::Simple(&v) => v,
            VoxelRef::Advanced(v) => v.as_trait().get_base_block(),
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

        Ok(self.get_voxel_internal(internal_coord))
    }

    pub fn get_voxel_internal(&self, coord: ICoordinate) -> VoxelRef {
        let id = match &self.grid {
            Chunk16Grid::B32(grid) => {
                let either = &grid[coord.x][coord.y][coord.z];
                match either {
                    ChunkB32Entry::Direct(block) => return VoxelRef::Simple(&block),
                    ChunkB32Entry::Mapped(idx) => idx.to_owned(),
                }
            },
            Chunk16Grid::B8(grid) => grid[coord.x][coord.y][coord.z] as u16,
            Chunk16Grid::B4(grid) => {
                let byte = grid[coord.x][coord.y][coord.z / 2] as u16;
                if coord.z % 2 == 0 {
                    byte & 0b1111
                } else {
                    byte >> 4
                }
            },
            Chunk16Grid::B2(grid) => {
                let byte = grid[coord.x][coord.y][coord.z / 4] as u16;
                let index_in_byte = coord.z % 4;
                let num_bit_shifts = index_in_byte * 2;
                (byte & (0b11 << num_bit_shifts)) >> num_bit_shifts
            },
        };

        self.get_palette(id).as_voxel_ref()
    }

    pub fn set_voxel_internal_base(&mut self, coord: ICoordinate, voxel: Voxel) {
        if let Chunk16Grid::B32(grid) = &mut self.grid {
            match voxel {
                Voxel::Simple(voxel) => {
                    grid[coord.x][coord.y][coord.z] = ChunkB32Entry::Direct(voxel)
                },
                Voxel::Advanced(_) => {
                    grid[coord.x][coord.y][coord.z] = ChunkB32Entry::Mapped(self.add_palette(voxel))
                },
            }
            todo!("check if palette may be removed");
            return;
        }

        let new_id = self.add_palette(voxel) as u8;
        if self.palette.len() > self.max_num_palettes() {
            self.upgrade();
        }

        match &mut self.grid {
            Chunk16Grid::B8(grid) => grid[coord.x][coord.y][coord.z] = new_id,
            Chunk16Grid::B4(grid) => {
                let byte_ref = &mut grid[coord.x][coord.y][coord.z / 2];
                let id_4bit = new_id & 0b1111;

                if coord.z % 2 == 0 {
                    *byte_ref &= 0b11110000;
                    *byte_ref |= id_4bit;
                } else {
                    *byte_ref &= 0b00001111;
                    *byte_ref |= id_4bit << 4;
                }
            },
            Chunk16Grid::B2(grid) => {
                let byte_ref = &mut grid[coord.x][coord.y][coord.z / 4];
                let index_in_byte = coord.z % 4;
                let num_bit_shifts = index_in_byte * 2;
                let id_2bit = new_id & 0b11;

                *byte_ref &= !(0b11 << num_bit_shifts);
                *byte_ref |= id_2bit << num_bit_shifts;
            },
            Chunk16Grid::B32(_) => panic!("Chunk16Grid::B32 should have been handled separately"),
        };
    }

    fn get_palette(&self, id: u16) -> &Voxel {
        for ele in self.palette {
            if ele.id == id {
                return &ele.element;
            }
        }

        unreachable!("Palette may never be empty")
    }

    fn add_palette(&mut self, voxel: Voxel) -> u16 {
        // first see if it is already in here
        for ele in &mut self.palette {
            if ele.element == voxel {
                ele.num_elements += 1;
                return ele.id;
            }
        }

        // otherwise, find the _last_ gap in the indices; we will use this index
        // (we find the last, not the first, because insertion is cheaper this way)
        for idx in (0..self.palette.len()).rev() {
            if self.palette[idx].id as usize != idx {
                self.palette.insert(
                    idx,
                    BlockMapping {
                        id: idx as u16,
                        num_elements: 1,
                        element: voxel,
                    },
                );

                return idx as u16;
            }
        }

        // no gap in the indices; append
        let new_id = self.palette.len() as u16;
        self.palette.push(BlockMapping {
            id: new_id,
            num_elements: 1,
            element: voxel,
        });
        new_id
    }

    fn max_num_palettes(&self) -> usize {
        match self.grid {
            Chunk16Grid::B32(_) => u32::MAX as usize,
            Chunk16Grid::B8(_) => (1 << 8) - 1,
            Chunk16Grid::B4(_) => (1 << 4) - 1,
            Chunk16Grid::B2(_) => (1 << 2) - 1,
        }
    }

    fn upgrade(&self) {
        todo!()
    }

    fn downgrade(&self) {
        todo!()
    }
}
