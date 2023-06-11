use std::collections::HashMap;

use sol_voxel_lib::{chunk::Chunk64, vector_alias::Coordinate};

pub struct World {
    pub chunks: HashMap<(i32, i32, i32), Box<Chunk64>>,
}

impl World {
    pub fn new() -> World {
        return World {
            chunks: HashMap::new(),
        };
    }

    pub fn get_relative<'s>(&'s self, coord : Coordinate) -> Option<&'s Chunk64> {
        self.chunks.get(&(coord.x, coord.y, coord.z)).map(|b| b.as_ref())
    }
}
