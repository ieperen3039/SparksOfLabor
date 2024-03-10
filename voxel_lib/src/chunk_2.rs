use crate::{vector_alias::Coordinate, voxel::{Voxel, VoxelRef}, voxel_errors::VoxelIndexError};

struct Map {
    id: u16,
    element : Voxel,
}

enum Chunk16 {
    B8(Chunk16b8),
    B4(Chunk16b4)
}

// 2^16 = 65536 different element types
// but there are only 4096 voxels per chunk
struct Chunk16b16 {
    grid: [[[u32; 16]; 16]; 16],
    palette: Vec<Map>,
}

// 2^8 = 256 different element types
struct Chunk16b8 {
    grid: [[[u8; 16]; 16]; 16],
    palette: Vec<Map>,
}

// 2^4 = 16 different element types
struct Chunk16b4 {
    grid: [[[u8; 8]; 16]; 16],
    palette: Vec<Map>,
}

impl Chunk16 {
    pub fn get_voxel(&self, coord: Coordinate) -> Result<VoxelRef, VoxelIndexError> {
        todo!()
    }
}