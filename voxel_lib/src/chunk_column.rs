use crate::chunk16::Chunk16;
use crate::vector_alias::{Coordinate16, ICoordinate};
use crate::voxel::VoxelRef;
use minecraft_protocol::components::blocks::BlockEntity;
use std::array::from_fn;

use minecraft_protocol::components::chunk as mc_chunk;
use serde::{Deserialize, Serialize};

// the protocol enforces 24 chunks per column.
// we may try to change this, but maybe the java client does not support anything else
pub const NUM_CHUNK_SECTIONS_PER_COLUMN: usize = 24;

type Heightmap = [[u16; 16]; 16];

pub enum WorldCommand {}

#[derive(Serialize, Deserialize)]
pub struct ChunkColumn {
    chunk_x_16: i32,
    chunk_z_16: i32,
    chunk_sections: [Chunk16; NUM_CHUNK_SECTIONS_PER_COLUMN],
    heightmap_motion_blocking: Heightmap,
    heightmap_world_surface: Heightmap,
}

pub struct ChunkColumnSerialized {
    pub chunk_x_16: i32,
    pub chunk_z_16: i32,
    pub chunk_sections: Vec<u8>,
    pub block_entities: Vec<BlockEntity>,
    pub heightmap_motion_blocking: Vec<i64>,
    pub heightmap_world_surface: Vec<i64>,
}

impl ChunkColumn {
    pub fn new(chunk_x_16: i32, chunk_z_16: i32) -> ChunkColumn {
        Self {
            chunk_x_16,
            chunk_z_16,
            chunk_sections: from_fn(|i| {
                Chunk16::new(
                    Coordinate16::new(chunk_x_16, i as i32, chunk_z_16),
                    minecraft_protocol::ids::blocks::Block::Air,
                )
            }),
            heightmap_motion_blocking: [[0; 16]; 16],
            heightmap_world_surface: [[0; 16]; 16],
        }
    }

    pub fn set_chunk(&mut self, y_16: i32, chunk: Chunk16) {
        self.update_heightmap(y_16, &chunk);

        self.chunk_sections[y_16 as usize] = chunk;
    }

    fn update_heightmap(&mut self, y_16: i32, chunk: &Chunk16) {
        let highest_chunk_value = (y_16 + 1) * 16;

        for z in 0..16usize {
            for x in 0..16usize {
                // we will assume that every surface is also blocking
                // "WORLD_SURFACE = All blocks other than air, cave air and void air"
                let mut blocking_found = false;
                let mut surface_found = false;

                if self.heightmap_motion_blocking[z][x] >= highest_chunk_value as u16 {
                    blocking_found = true;
                }

                if self.heightmap_world_surface[z][x] >= highest_chunk_value as u16 {
                    surface_found = true;
                }

                if blocking_found && surface_found {
                    continue;
                }

                for y in (0..16usize).rev() {
                    let block = chunk
                        .get_voxel_internal(ICoordinate::new(x, y, z))
                        .get_block();

                    if !blocking_found {
                        if !block.is_blocking() {
                            continue;
                        }
                        // first blocking block of the pillar
                        self.heightmap_motion_blocking[z][x] = ((y_16 * 16) as u16) + (y as u16);

                        if surface_found {
                            break;
                        }
                        blocking_found = true;
                    } else if block.is_air_block() {
                        self.heightmap_world_surface[z][x] = ((y_16 * 16) as u16) + (y as u16);
                        break;
                    }
                }
            }
        }
    }

    pub fn to_minecraft(&self) -> Result<ChunkColumnSerialized, &'static str> {
        let results = self
            .chunk_sections
            .iter()
            .map(Chunk16::to_minecraft)
            .map(|(chunks, _, block_entities)| (chunks, block_entities))
            .unzip();

        let chunk_sections = results.0;
        let block_entities: Vec<Vec<BlockEntity>> = results.1;

        let chunk_sections_serialized: Vec<u8> = mc_chunk::Chunk::into_data(chunk_sections)?;

        let block_entities: Vec<BlockEntity> = block_entities.into_iter().flatten().collect();

        let world_surface = self.heightmap_to_minecraft(&self.heightmap_world_surface);
        let motion_blocking = self.heightmap_to_minecraft(&self.heightmap_motion_blocking);

        Ok(ChunkColumnSerialized {
            chunk_x_16: self.chunk_x_16,
            chunk_z_16: self.chunk_z_16,
            chunk_sections: chunk_sections_serialized,
            block_entities,
            heightmap_motion_blocking: motion_blocking,
            heightmap_world_surface: world_surface,
        })
    }

    fn heightmap_to_minecraft(&self, heightmap: &Heightmap) -> Vec<i64> {
        let world_height = NUM_CHUNK_SECTIONS_PER_COLUMN * 16;
        let bits_per_element = usize::BITS - world_height.leading_zeros();

        let mut result = Vec::new();
        let mut accumulator: u64 = 0;
        let mut start_bit_index = 0;
        for x in 0..16 {
            for z in 0..16 {
                debug_assert!(heightmap[z][x] < (1 << bits_per_element));

                accumulator |= (heightmap[z][x] as u64) << start_bit_index;
                start_bit_index += bits_per_element;

                if start_bit_index > u64::BITS {
                    result.push(accumulator as i64);
                    accumulator = 0;
                    start_bit_index = 0;
                }
            }
        }

        return result;
    }

    fn heightmap_from_minecraft(&self, byte_array: &Vec<i64>) -> Heightmap {
        // could be a constant
        let world_height = self.chunk_sections.len() * 16;
        let bits_per_element = usize::BITS - world_height.leading_zeros();

        let mut heightmap = [[0; 16]; 16];
        let mask: u64 = (1 << bits_per_element) - 1;
        let mut byte_index = 0;
        let mut start_bit_index = 0;
        for x in 0..16 {
            for z in 0..16 {
                // check first for the sake of the trailing assert.
                // note that the unnecessary extra check replaces a possibly unnecessary increment
                if start_bit_index > u64::BITS {
                    byte_index += 1;
                    start_bit_index = 0;
                }

                let byte = byte_array[byte_index] as u64;
                heightmap[z][x] = ((byte >> start_bit_index) & mask) as u16;
                start_bit_index += bits_per_element;
            }
        }

        assert_eq!(byte_array.len(), byte_index);

        return heightmap;
    }

    pub fn for_each<Action: FnMut(&ICoordinate, VoxelRef)>(&self, mut action: Action) {
        // I regret nothing
        for y16 in 0..24usize {
            let chunk16 = &self.chunk_sections[y16];
            let index_vector_16_base = ICoordinate::new(0, y16 * 16, 0);
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
