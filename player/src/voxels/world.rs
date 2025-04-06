use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use sol_voxel_lib::{chunk::Chunk64, chunk_column::ChunkColumn, vector_alias::*};

pub struct World {
    chunks: HashMap<Coordinate64, Box<Chunk64>>,
}

impl World {
    pub fn new() -> World {
        let mut chunks = HashMap::new();

        for z in -5..5 {
            for x in -5..5 {
                for y in 0..4 {
                    let coord = Coordinate64::new(x, y, z);
                    chunks.insert(coord, Box::from(Chunk64::new(coord, minecraft_protocol::ids::blocks::Block::Stone)));
                }

                for y in 4..16 {
                    let coord = Coordinate64::new(x, y, z);
                    chunks.insert(coord, Box::from(Chunk64::new(coord, minecraft_protocol::ids::blocks::Block::Air)));
                }
            }
        }

        World { chunks }
    }

    pub fn get_chunk<'s>(&'s self, coord: Coordinate64) -> Option<&'s Chunk64> {
        self.chunks.get(&coord).map(Box::as_ref)
    }

    pub fn get_area(&self, player_position: Position) -> Vec<&ChunkColumn> {
        let chunk_column = Vec::new();

        // TODO

        return chunk_column;
    }
}
