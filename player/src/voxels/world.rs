use minecraft_protocol::data::block_states::BlockWithState;
use sol_voxel_lib::chunk16::Chunk16;
use sol_voxel_lib::{chunk_column::ChunkColumn, vector_alias::*};
use std::collections::HashMap;

pub struct World {
    chunks: HashMap<ChunkColumnCoordinate, Box<ChunkColumn>>,
}

impl World {
    /**
     * Creates a world initialized with a 21x21 chunk square stone floor around y=0
     */
    pub fn new() -> World {
        let mut chunks = HashMap::new();

        for z in -10..=10 {
            for x in -10..=10 {
                let mut chunk_column = Box::from(ChunkColumn::new(x, z));

                // -64 to 0
                for y in 0..4 {
                    chunk_column.set_chunk(y, Chunk16::new(
                        Coordinate16::new(x, y, z),
                        minecraft_vanilla::ids::blocks::BlockId::Stone.into(),
                        false
                    ), todo!());
                }

                chunks.insert(
                    ChunkColumnCoordinate { x, z },
                    chunk_column,
                );
            }
        }

        World { chunks }
    }

    pub fn get_chunk(&self, coord: &ChunkColumnCoordinate) -> Option<&ChunkColumn> {
        self.chunks.get(coord).map(Box::as_ref)
    }

    pub fn get_area(&self, player_position: Position) -> Vec<&ChunkColumn> {
        let mut area = Vec::new();

        let center_point = ChunkColumnCoordinate::containing_position(&player_position);

        for z in -5..5 {
            for x in -5..5 {
                let chunk_coord = center_point.add(x, z);
                match self.get_chunk(&chunk_coord) {
                    None => {},
                    Some(chunk_column) => area.push(chunk_column),
                };
            }
        }

        return area;
    }

    pub fn get_block(&self, coord: Coordinate) -> Option<BlockWithState> {
        let center_point = ChunkColumnCoordinate::containing_coord(&coord);
        let chunk = self.get_chunk(&center_point);
        if let Some(chunk) = chunk {
            chunk.
        } else {
            None
        }
    }
}
