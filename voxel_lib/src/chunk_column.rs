
use crate::chunk::Chunk16;

type Heightmap = [[u16; 16]; 16];

pub enum WorldCommand {}

pub struct ChunkColumn {
    pub chunk_x_16: i32,
    pub chunk_y_16: i32,
    pub chunk_sections: Vec<Chunk16>,
    pub heightmap_motion_blocking: Heightmap,
    pub heightmap_world_surface: Heightmap,
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
