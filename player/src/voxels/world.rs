use std::collections::HashMap;

use sol_voxel_lib::chunk16::Chunk16;
use sol_voxel_lib::{chunk_column::ChunkColumn, vector_alias::*};

pub struct World {
    chunks: HashMap<ChunkColumnCoordinate, Box<ChunkColumn>>,
}

impl World {
    /**
     * Creates a world initialized with an 11x11 chunk square stone floor around y=64, with a world height of 256
     */
    pub fn new() -> World {
        let mut chunks = HashMap::new();

        for z in -5..5 {
            for x in -5..5 {
                let mut chunk_sections = Vec::new();
                for y in 0..16 {
                    chunk_sections.push(Chunk16::new(
                        Coordinate16::new(x, y, z),
                        minecraft_protocol::ids::blocks::Block::Stone,
                    ));
                }

                chunks.insert(
                    ChunkColumnCoordinate { x, z },
                    Box::from(ChunkColumn::new(x, z, chunk_sections)),
                );
            }
        }

        World { chunks }
    }

    pub fn get_chunk(&self, coord: &ChunkColumnCoordinate) -> Option<&ChunkColumn> {
        self.chunks.get(coord).map(Box::as_ref)
    }

    pub fn get_area(&self, player_position: Position) -> Vec<&ChunkColumn> {
        let chunk_column = Vec::new();

        self.chunks.get(&ChunkColumnCoordinate::containing_position(
            &player_position,
        ));

        return chunk_column;
    }
}
