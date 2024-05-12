use std::collections::HashMap;

use crate::{chunk::Chunk64, vector_alias::Coordinate64};

pub struct World {
    pub chunks: HashMap<(i32, i32, i32), Box<Chunk64>>,
}

impl World {
    pub fn new() -> World {
        return World {
            chunks: HashMap::new(),
        };
    }

    pub fn get_chunk<'s>(&'s self, coord : Coordinate64) -> Option<&'s Chunk64> {
        self.chunks.get(&(coord.x, coord.y, coord.z)).map(|b| b.as_ref())
    }
    
    pub fn get_area(&self, player_position: nalgebra::OPoint<f32, nalgebra::Const<3>>) -> (HashMap<String, minecraft_protocol::nbt::NbtTag>, ChunkColumn) {
        todo!()
    }
}