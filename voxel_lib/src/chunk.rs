use minecraft_protocol::components::blocks::BlockEntity;
use serde::{Deserialize, Serialize};

use crate::palette::Palette;
use crate::vector_alias::{coordinate16_to_absolute, Coordinate16};
use crate::voxel::{Voxel, VoxelRef};
use crate::{
    vector_alias::{Coordinate, ICoordinate},
    voxel_errors::VoxelIndexError,
};
use minecraft_protocol::{
    components::{blocks as mc_blocks, chunk as mc_chunk},
    ids::blocks as mc_ids,
};

const CHUNK_B32_ENTRY_FLAG_BIT: u32 = 1 << 31;

// compressed 2-ple enum using the 32th bit as a flag.
// is a 31-bit
#[derive(Serialize, Deserialize, Clone, Copy)]
struct ChunkB32Entry {
    value: u32,
}

impl ChunkB32Entry {
    pub fn new_empty() -> ChunkB32Entry {
        ChunkB32Entry { value: 0 }
    }

    pub fn make_direct(value: u32) -> ChunkB32Entry {
        assert!(value & CHUNK_B32_ENTRY_FLAG_BIT == 0);
        ChunkB32Entry { value }
    }

    pub fn make_mapped(mapped_id: u16) -> ChunkB32Entry {
        let mapped_id = mapped_id as u32;
        ChunkB32Entry {
            value: mapped_id | CHUNK_B32_ENTRY_FLAG_BIT,
        }
    }

    pub fn is_direct(&self) -> bool {
        (self.value & CHUNK_B32_ENTRY_FLAG_BIT) == 0
    }

    pub fn is_mapped(&self) -> bool {
        !self.is_direct()
    }

    pub fn as_direct(&self) -> u32 {
        self.value
    }

    pub fn as_mapped(&self) -> u16 {
        // mask off at least CHUNK_B32_ENTRY_FLAG_BIT
        (self.value & 0xFFFF) as u16
    }
}

#[derive(Serialize, Deserialize)]
pub struct Chunk64 {
    voxels: [[[Chunk16; 4]; 4]; 4],
    zero_coordinate: Coordinate,
}

#[derive(Serialize, Deserialize)]
pub struct Chunk16 {
    // NOTE: [y][z][x] where y is height
    grid: Chunk16Grid,
    palette: Palette,
    biomes: [[[u8; 4]; 4]; 4],
    zero_coordinate: Coordinate,
    num_non_air_blocks: u16,
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
    B0,
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
        for y16 in 0..4usize {
            for z16 in 0..4usize {
                for x16 in 0..4usize {
                    let chunk16 = &self.voxels[y16][z16][x16];
                    let index_vector_16_base = ICoordinate::new(x16, y16, z16) * 16;
                    for y in 0..16usize {
                        for z in 0..16usize {
                            for x in 0..16usize {
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
    pub fn new(location: Coordinate16, fill_value: mc_ids::Block) -> Chunk16 {
        let fill_voxel = Voxel::from_block(fill_value);
        let is_air = fill_voxel.is_air();

        Chunk16 {
            grid: Chunk16Grid::B0,
            palette: Palette::fill(fill_value.id()),
            biomes: Default::default(),
            zero_coordinate: coordinate16_to_absolute(location),
            num_non_air_blocks: if is_air { 0 } else { 16 * 16 * 16 },
        }
    }

    pub fn get_voxel(&self, coord: Coordinate) -> Result<VoxelRef, VoxelIndexError> {
        let internal_coord = to_internal(coord, self.zero_coordinate, 16, 16)
            .ok_or(VoxelIndexError { coordinate: coord })?;

        Ok(self.get_voxel_internal(internal_coord))
    }

    pub fn get_voxel_internal(&self, coord: ICoordinate) -> VoxelRef {
        let id = match &self.grid {
            Chunk16Grid::B32(grid) => {
                let entry = grid[coord.y][coord.z][coord.x];
                if entry.is_direct() {
                    return VoxelRef::Inferred(entry.as_direct());
                };
                assert!(entry.is_mapped());
                entry.as_mapped()
            },
            Chunk16Grid::B8(grid) => grid[coord.y][coord.z][coord.x] as u16,
            Chunk16Grid::B4(grid) => {
                let byte = grid[coord.y][coord.z][coord.x / 2] as u16;
                if coord.x % 2 == 0 {
                    byte & 0b1111
                } else {
                    byte >> 4
                }
            },
            Chunk16Grid::B2(grid) => {
                let byte = grid[coord.y][coord.z][coord.x / 4] as u16;
                let index_in_byte = coord.x % 4;
                let num_bit_shifts = index_in_byte * 2;
                (byte & (0b11 << num_bit_shifts)) >> num_bit_shifts
            },
            Chunk16Grid::B0 => 0,
        };

        return self.palette.get(id);
    }

    pub fn set_voxel_internal(&mut self, coord: ICoordinate, voxel: Voxel) {
        let voxel_block_id = voxel.get_block_id();
        let voxel_is_simple = voxel.is_simple();

        if !voxel.is_air() {
            self.num_non_air_blocks += 1;
        }

        let new_id = self.palette.add_voxel(voxel, coord);
        if self.palette.len() > self.max_num_palettes() {
            self.upgrade();
        }

        let old_id = match &mut self.grid {
            Chunk16Grid::B8(grid) => {
                let old_id = grid[coord.y][coord.z][coord.x];
                grid[coord.y][coord.z][coord.x] = new_id as u8;
                old_id as u16
            },
            Chunk16Grid::B4(grid) => {
                let byte_ref = &mut grid[coord.y][coord.z][coord.x / 2];
                let old_id: u8;
                let id_4bit = new_id as u8 & 0b1111;

                if coord.x % 2 == 0 {
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
                let byte_ref = &mut grid[coord.y][coord.z][coord.x / 4];
                let index_in_byte = coord.x % 4;
                let num_bit_shifts = index_in_byte * 2;
                let id_2bit = new_id as u8 & 0b11;

                let old_id = (*byte_ref & (0b11 << num_bit_shifts)) >> num_bit_shifts;

                *byte_ref &= !(0b11 << num_bit_shifts);
                *byte_ref |= id_2bit << num_bit_shifts;

                old_id as u16
            },
            Chunk16Grid::B32(grid) => {
                let chunk_b32_entry = grid[coord.y][coord.z][coord.x];
                let old_id = if chunk_b32_entry.is_direct() {
                    self.palette
                        .find(chunk_b32_entry.as_direct())
                        .expect("direct entries in grid must be in palette")
                } else {
                    chunk_b32_entry.as_mapped()
                };

                if voxel_is_simple {
                    grid[coord.y][coord.z][coord.x] = ChunkB32Entry::make_direct(voxel_block_id);
                } else {
                    grid[coord.y][coord.z][coord.x] = ChunkB32Entry::make_mapped(new_id);
                }

                if self.palette.len() < self.min_num_palettes() {
                    self.downgrade();
                }

                old_id
            },
            Chunk16Grid::B0 => 0,
        };

        let old_block_id = self.palette.remove(old_id);

        let voxel_was_air = mc_ids::Block::from_id(old_block_id)
            .expect("invalid id")
            .is_air_block();

        if !voxel_was_air {
            // we removed a non-air block
            self.num_non_air_blocks -= 1;
        }
    }

    pub fn from_minecraft(mc_chunk: &mc_chunk::Chunk, position: Coordinate16) -> Chunk16 {
        match &mc_chunk.blocks {
            mc_chunk::PalettedData::Paletted { palette, indexed } => {
                Chunk16::from_raw_palette(indexed, palette, position)
            },
            mc_chunk::PalettedData::Single { value } => {
                // TODO from_id or from_state_id?
                Chunk16::new(
                    position,
                    mc_ids::Block::from_id(*value).expect("unkown block id"),
                )
            },
            mc_chunk::PalettedData::Raw { values } => Chunk16::from_direct(values, position),
        }
    }

    pub fn to_minecraft(
        sol_chunk: &Self,
    ) -> (mc_chunk::Chunk, Coordinate16, Vec<mc_blocks::BlockEntity>) {
        let mut indices = Vec::new();
        let mut palette = Vec::new();

        match &sol_chunk.grid {
            Chunk16Grid::B8(_) | Chunk16Grid::B4(_) | Chunk16Grid::B2(_) => {
                indices.reserve(16 * 16 * 16);

                for ele in sol_chunk.palette.all() {
                    palette.push(ele);
                }
            },
            Chunk16Grid::B32(_) | Chunk16Grid::B0 => {},
        }

        let block_entities: Vec<mc_blocks::BlockEntity> = sol_chunk
            .palette
            .all_nbt_voxels()
            .iter()
            .map(|v| {
                BlockEntity::new(
                    v.relative_x as u8,
                    sol_chunk.zero_coordinate.y + (v.relative_y as i32),
                    v.relative_z as u8,
                    v.voxel.get_block(),
                    v.voxel.get_nbt_data(),
                )
            })
            .collect();

        let blocks = match &sol_chunk.grid {
            Chunk16Grid::B32(grid) => {
                let mut values = Vec::new();
                values.reserve(16 * 16 * 16);

                for y in 0..16usize {
                    for z in 0..16usize {
                        for x in 0..16usize {
                            let elt = grid[y][z][x];

                            if elt.is_mapped() {
                                let voxel = &sol_chunk.palette.get(elt.as_mapped());
                                values.push(voxel.get_block_id());
                            } else {
                                values.push(elt.as_direct());
                            }
                        }
                    }
                }
                mc_chunk::PalettedData::Raw { values }
            },
            Chunk16Grid::B8(grid) => {
                for y in 0..16usize {
                    for z in 0..16usize {
                        for x in 0..16usize {
                            indices.push(grid[y][z][x]);
                        }
                    }
                }
                mc_chunk::PalettedData::Paletted {
                    palette,
                    indexed: indices,
                }
            },
            Chunk16Grid::B4(grid) => {
                for y in 0..16usize {
                    for z in 0..16usize {
                        // 0..8 because we grab the whole byte
                        for x2 in 0..8usize {
                            let byte = grid[y][z][x2];

                            let id = byte & 0b00001111;
                            indices.push(id);

                            let id = (byte & 0b11110000) >> 4;
                            indices.push(id);
                        }
                    }
                }
                mc_chunk::PalettedData::Paletted {
                    palette,
                    indexed: indices,
                }
            },
            Chunk16Grid::B2(grid) => {
                for y in 0..16usize {
                    for z in 0..16usize {
                        // 0..4 because we grab the full byte
                        for x4 in 0..4usize {
                            let byte = grid[y][z][x4];
                            for index_in_byte in 0..4 {
                                let num_bit_shifts = index_in_byte * 2;
                                let masked_value =
                                    (byte & (0b11 << num_bit_shifts)) >> num_bit_shifts;
                                indices.push(masked_value);
                            }
                        }
                    }
                }

                mc_chunk::PalettedData::Paletted {
                    palette,
                    indexed: indices,
                }
            },
            Chunk16Grid::B0 => {
                let voxel = &sol_chunk.palette.get(0);
                mc_chunk::PalettedData::Single {
                    value: voxel.get_block_id(),
                }
            },
        };

        return (
            mc_chunk::Chunk {
                block_count: sol_chunk.num_non_air_blocks as i16,
                blocks,
                biomes: mc_chunk::PalettedData::Single { value: 0 },
            },
            sol_chunk.get_zero_coordinate() / 16,
            block_entities,
        );
    }

    fn from_raw_palette(grid: &[u8], palette: &[u32], position: Coordinate16) -> Chunk16 {
        let mut sol_palette = Palette::new();
        for id in palette {
            sol_palette.add(*id);
        }

        let grid = if palette.len() < (1 << 2) {
            Chunk16::from_slice_b2(grid)
        } else if palette.len() < (1 << 4) {
            Chunk16::from_slice_b4(grid)
        } else {
            // it cannot be worse than 8-bit, because the input grid is 8-bit
            Chunk16::from_slice_b8(grid)
        };

        return Chunk16 {
            grid,
            palette: sol_palette,
            biomes: [[[0; 4]; 4]; 4],
            zero_coordinate: coordinate16_to_absolute(position),
            num_non_air_blocks: 0,
        };
    }

    fn from_slice_b2(slice: &[u8]) -> Chunk16Grid {
        let mut grid = Box::new([[[0; 4]; 16]; 16]);

        let mut i = 0;
        // we assume that the slice can be indexed: slice[y][z][x]
        for y in 0..16usize {
            for z in 0..16usize {
                // 0..4 because we grab the full byte
                for x4 in 0..4usize {
                    let byte_ref = &mut grid[y][z][x4];
                    for index_in_byte in 0..4 {
                        let new_id = slice[i];

                        let num_bit_shifts = index_in_byte * 2;
                        let id_2bit = new_id & 0b11;

                        *byte_ref &= !(0b11 << num_bit_shifts);
                        *byte_ref |= id_2bit << num_bit_shifts;

                        i += 1;
                    }
                }
            }
        }

        return Chunk16Grid::B2(grid);
    }

    fn from_slice_b4(slice: &[u8]) -> Chunk16Grid {
        let mut grid = Box::new([[[0; 8]; 16]; 16]);

        let mut i = 0;
        for y in 0..16usize {
            for z in 0..16usize {
                for x2 in 0..8usize {
                    // 0..8 because we grab the full byte
                    let byte_ref = &mut grid[y][z][x2];
                    for index_in_byte in 0..2 {
                        let new_id = slice[i];
                        let id_4bit = new_id & 0b1111;

                        if index_in_byte == 0 {
                            *byte_ref &= 0b11110000;
                            *byte_ref |= id_4bit;
                        } else {
                            *byte_ref &= 0b00001111;
                            *byte_ref |= id_4bit << 4;
                        }

                        i += 1;
                    }
                }
            }
        }

        return Chunk16Grid::B4(grid);
    }

    fn from_slice_b8(slice: &[u8]) -> Chunk16Grid {
        let mut grid = Box::new([[[0; 16]; 16]; 16]);

        let mut i = 0;
        for y in 0..16usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    grid[y][z][x] = slice[i];
                    i += 1;
                }
            }
        }

        return Chunk16Grid::B8(grid);
    }

    fn from_slice_b32(slice: &[u8]) -> Chunk16Grid {
        let mut grid = Box::new([[[ChunkB32Entry::new_empty(); 16]; 16]; 16]);

        let mut i = 0;
        for y in 0..16usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    grid[y][z][x] = ChunkB32Entry::make_mapped(slice[i] as u16);
                    i += 1;
                }
            }
        }

        return Chunk16Grid::B32(grid);
    }

    fn from_direct(slice: &[u32], position: Coordinate16) -> Chunk16 {
        // we assume that this will require a 32-bit mapping, because otherwise it would be an indirect map
        let mut grid = Box::new([[[ChunkB32Entry::new_empty(); 16]; 16]; 16]);

        let mut i = 0;
        for y in 0..16usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    grid[y][z][x] = ChunkB32Entry::make_direct(slice[i]);
                    i += 1;
                }
            }
        }

        return Chunk16 {
            grid: Chunk16Grid::B32(grid),
            palette: Palette::new(),
            biomes: [[[0; 4]; 4]; 4],
            zero_coordinate: coordinate16_to_absolute(position),
            num_non_air_blocks: 0,
        };
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

    fn upgrade(&mut self) {
        let mut new_palette = Palette::new();

        match &self.grid {
            Chunk16Grid::B32(_) => return,
            Chunk16Grid::B8(grid) => {
                let mut new_grid = Box::new([[[ChunkB32Entry::new_empty(); 16]; 16]; 16]);

                for y in 0..16usize {
                    for z in 0..16usize {
                        for x in 0..16usize {
                            let id = grid[y][z][x];
                            let block_mapping = self.palette.get(id as u16);

                            // note: we do not change the palette
                            if block_mapping.is_simple() {
                                new_grid[y][z][x] = ChunkB32Entry::make_direct(id as u32);
                            } else {
                                new_grid[y][z][x] = ChunkB32Entry::make_mapped(id as u16);
                            }
                        }
                    }
                }

                self.grid = Chunk16Grid::B32(new_grid)
            },
            Chunk16Grid::B4(grid) => {
                let mut new_grid = Box::new([[[0; 16]; 16]; 16]);

                for y in 0..16usize {
                    for z in 0..16usize {
                        for x in 0..16usize {
                            let byte_ref: u8 = grid[y][z][x / 2];

                            if x % 2 == 0 {
                                new_grid[y][z][x] = byte_ref;
                            } else {
                                new_grid[y][z][x] = (byte_ref & 0b11110000) >> 4;
                            };
                        }
                    }
                }

                self.grid = Chunk16Grid::B8(new_grid)
            },
            Chunk16Grid::B2(grid) => {
                let mut new_grid = Box::new([[[0; 8]; 16]; 16]);

                for y in 0..16usize {
                    for z in 0..16usize {
                        for x in 0..16usize {
                            let old_byte: u8 = grid[y][z][x / 4];
                            let index_in_byte = x % 4;
                            let num_bit_shifts = index_in_byte * 2;
                            let id = (old_byte & (0b11 << num_bit_shifts)) >> num_bit_shifts;

                            let byte_ref: &mut u8 = &mut new_grid[y][z][x / 2];

                            if x % 2 == 0 {
                                *byte_ref |= id;
                            } else {
                                *byte_ref |= id << 4;
                            }
                        }
                    }
                }

                self.grid = Chunk16Grid::B4(new_grid)
            },
            Chunk16Grid::B0 => {
                // fill with id 0, because that is always the only index that we had in use
                self.grid = Chunk16Grid::B2(Box::new([[[0; 4]; 16]; 16]))
            },
        };
    }

    fn downgrade(&self) {
        todo!()
    }

    pub fn get_zero_coordinate(&self) -> Coordinate {
        self.zero_coordinate
    }
}
