use std::collections::HashMap;

use sol_voxel_lib::chunk::Chunk64;

pub struct World {
    pub chunks: HashMap<(i32, i32, i32), Box<Chunk64>>,
}

impl World {
    pub fn new() -> World {
        return World {
            chunks: HashMap::new(),
        };
    }
}
