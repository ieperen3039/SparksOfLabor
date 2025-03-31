use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    chunk::{Chunk16, Chunk64},
    vector_alias::{Coordinate, Coordinate16, Position},
};

type Heightmap = [[u16; 16]; 16];

pub enum WorldCommand {}

pub struct ChunkColumn {
    pub chunk_x_16: i32,
    pub chunk_y_16: i32,
    pub chunk_sections: Vec<Chunk16>,
    pub heightmap_motion_blocking: Heightmap,
    pub heightmap_world_surface: Heightmap,
}
pub struct World {
    queue_input: Sender<WorldCommand>,
    queue: Receiver<WorldCommand>,
    chunks: HashMap<Coordinate16, Box<Chunk64>>,
}

impl World {
    pub fn new() -> World {
        let (sender, receiver) = std::sync::mpsc::channel();
        return World {
            queue_input: sender,
            queue: receiver,
            chunks: HashMap::new(),
        };
    }

    pub fn get_message_queue(&self) -> Sender<WorldCommand> {
        self.queue_input.clone()
    }

    pub fn get_chunk<'s>(&'s self, coord: Coordinate16) -> Option<&'s Chunk64> {
        self.chunks.get(&coord).map(Box::as_ref)
    }

    pub fn get_area(&self, player_position: Position) -> Vec<&ChunkColumn> {
        let chunk_column = Vec::new();

        // TODO

        return chunk_column;
    }
}

impl ChunkColumn {
    pub fn to_minecraft(&self, heightmap : &Heightmap) -> Vec<i64> {
        let world_height = self.chunk_sections.len() * 16;
        let bits_per_element = usize::BITS - world_height.leading_zeros();

        let mut result = Vec::new();
        let mut accumulator: u64 = 0;
        let mut start_bit_index = 0;
        for x in 0..16 {
            for z in 0..16 {
                debug_assert!(heightmap[z][x] < (1 << bits_per_element));

                accumulator |= (heightmap[z][x] >> start_bit_index) as u64;
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

    pub fn from_minecraft(&self, byte_array : &Vec<i64>) -> Heightmap {
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
                heightmap[z][x] = ((byte << start_bit_index) & mask) as u16;
                start_bit_index += bits_per_element;
            }
        }

        assert!(byte_array.len() == byte_index);

        return heightmap;
    }
}
