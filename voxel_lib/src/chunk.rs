use minecraft_protocol::ids::blocks::Block;
use minecraft_protocol::nbt::NbtTag;
use serde::{Deserialize, Serialize};

use crate::voxel::{self, Voxel, VoxelRef};
use crate::{
    block::BaseVoxel,
    vector_alias::{Coordinate, ICoordinate},
    voxel_errors::VoxelIndexError,
};

#[derive(Serialize, Deserialize, Clone, Copy)]
enum ChunkB32Entry {
    Direct(u32),
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
    block_type: Voxel,
}

#[derive(Serialize, Deserialize)]
struct Palette 
{
    mapping: Vec<BlockMapping>,
}

#[derive(Serialize, Deserialize)]
pub struct Chunk16 {
    grid: Chunk16Grid,
    palette: Palette,
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
    // 1 element type
    B0
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

    pub fn for_each<Action: FnMut(&ICoordinate, VoxelRef)>(&self, mut action: Action) {
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
                                let voxel = chunk16.get_voxel_internal(index_vector);

                                let coord = index_vector_16_base + index_vector;
                                action(&coord, voxel);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Chunk16 {
    pub fn get_voxel(&self, coord: Coordinate) -> Result<VoxelRef, VoxelIndexError> {
        let internal_coord = to_internal(coord, self.zero_coordinate, 16, 16)
            .ok_or(VoxelIndexError { coordinate: coord })?;

        Ok(self.get_voxel_internal(internal_coord))
    }

    pub fn get_voxel_internal(&self, coord: ICoordinate) -> VoxelRef {
        let id = match &self.grid {
            Chunk16Grid::B32(grid) => {
                let either = grid[coord.x][coord.y][coord.z];
                match either {
                    ChunkB32Entry::Direct(block) => return VoxelRef::Inferred(block),
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
            Chunk16Grid::B0 => 0,
        };

        return VoxelRef::Real(&self.palette.get(id).block_type);
    }

    pub fn set_voxel_internal_base(&mut self, coord: ICoordinate, voxel: Voxel) {
        let voxel_block_id = voxel.get_block_id();
        let voxel_is_simple = voxel.is_simple();

        let new_id = self.palette.add(voxel);
        if self.palette.len() > self.max_num_palettes() {
            self.upgrade();
        }

        let old_id = match &mut self.grid {
            Chunk16Grid::B8(grid) => {
                let old_id = grid[coord.x][coord.y][coord.z];
                grid[coord.x][coord.y][coord.z] = new_id as u8;
                old_id as u16
            },
            Chunk16Grid::B4(grid) => {
                let byte_ref = &mut grid[coord.x][coord.y][coord.z / 2];
                let old_id: u8;
                let id_4bit = new_id as u8 & 0b1111;
 
                if coord.z % 2 == 0 {
                    old_id = *byte_ref & 0b00001111;
                    *byte_ref &= 0b11110000;
                    *byte_ref |= id_4bit;
                } else {
                    old_id = (*byte_ref & 0b11110000) >> 4;
                    *byte_ref &= 0b00001111;
                    *byte_ref |= id_4bit << 4;
                }

                old_id as u16
            },
            Chunk16Grid::B2(grid) => {
                let byte_ref = &mut grid[coord.x][coord.y][coord.z / 4];
                let index_in_byte = coord.z % 4;
                let num_bit_shifts = index_in_byte * 2;
                let id_2bit = new_id as u8 & 0b11;

                let old_id = (*byte_ref & (0b11 << num_bit_shifts)) >> num_bit_shifts;

                *byte_ref &= !(0b11 << num_bit_shifts);
                *byte_ref |= id_2bit << num_bit_shifts;

                old_id as u16
            },
            Chunk16Grid::B32(grid) => {
                let old_id = match grid[coord.x][coord.y][coord.z] {
                    ChunkB32Entry::Mapped(old_value) => old_value,
                    ChunkB32Entry::Direct(block_id) => self.palette.find(block_id),
                };

                if voxel_is_simple {
                    grid[coord.x][coord.y][coord.z] = ChunkB32Entry::Direct(voxel_block_id);
                } else {
                    grid[coord.x][coord.y][coord.z] = ChunkB32Entry::Mapped(new_id);
                }

                // TODO calculate if downgrade is necessary

                old_id
            },
            Chunk16Grid::B0 => 0,
        };

        self.palette.remove(old_id);
    }

    fn max_num_palettes(&self) -> usize {
        match self.grid {
            Chunk16Grid::B32(_) => u32::MAX as usize,
            Chunk16Grid::B8(_) => (1 << 8) - 1,
            Chunk16Grid::B4(_) => (1 << 4) - 1,
            Chunk16Grid::B2(_) => (1 << 2) - 1,
            Chunk16Grid::B0 => 1,
        }
    }

    fn min_num_palettes(&self) -> usize {
        // we allow a bigger representation, even if the smaller rep has this number of empty spots
        // to prevent us from converting chunks all the time
        const PALETTE_HYSTERESIS_VALUE: usize = 1;

        let max_of_downgrade = match self.grid {
            Chunk16Grid::B32(_) => (1 << 8) - 1,
            Chunk16Grid::B8(_) => (1 << 4) - 1,
            Chunk16Grid::B4(_) => (1 << 2) - 1,
            Chunk16Grid::B2(_) => 1,
            Chunk16Grid::B0 => 0,
        };

        if max_of_downgrade <= PALETTE_HYSTERESIS_VALUE {
            return 0;
        } else {
            return max_of_downgrade - PALETTE_HYSTERESIS_VALUE;
        }
    }

    fn upgrade(&self) {
        todo!()
    }

    fn downgrade(&self) {
        todo!()
    }
}

impl Palette {
    fn add(&mut self, voxel: Voxel) -> u16 {
        // first see if it is already in here
        for ele in &mut self.mapping {
            if ele.block_type == voxel {
                ele.num_elements += 1;
                return ele.id;
            }
        }

        // otherwise, find the _last_ gap in the indices; we will use this index
        // (we find the last, not the first, because insertion is cheaper this way)
        for idx in (0..self.mapping.len()).rev() {
            if self.mapping[idx].id as usize != idx {
                self.mapping.insert(
                    idx,
                    BlockMapping {
                        id: idx as u16,
                        num_elements: 1,
                        block_type: voxel,
                    },
                );

                return idx as u16;
            }
        }

        // no gap in the indices; append
        let new_id = self.mapping.len() as u16;
        self.mapping.push(BlockMapping {
            id: new_id,
            num_elements: 1,
            block_type: voxel,
        });

        return new_id;
    }

    fn remove(&mut self, id: u16) {
        let index = self
            .mapping
            .iter()
            .position(|ele| ele.id == id)
            .expect("remove_palette: Id does not exit");
        self.mapping[index].num_elements -= 1;
        self.mapping.remove(index);
    }

    fn get(&self, id: u16) -> &BlockMapping {
        for ele in &self.mapping {
            if ele.id == id {
                return &ele;
            }
        }

        unreachable!("Id not found")
    }

    fn find(&self, block_id: u32) -> u16 {
        for ele in &self.mapping {
            if ele.block_type.get_block_id() == block_id {
                return ele.id;
            }
        }

        unreachable!("Id not found")
    }
    
    fn len(&self) -> usize {
        self.mapping.len()
    }
}