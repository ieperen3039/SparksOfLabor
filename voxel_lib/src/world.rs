use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    chunk::{Chunk16, Chunk64},
    vector_alias::{Coordinate, Coordinate16, Position},
};

pub enum WorldCommand {}

pub struct ChunkColumn {
    pub chunk_x_16: i32,
    pub chunk_y_16: i32,
    pub chunk_sections: Vec<Chunk16>,
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

    pub fn get_area(
        &self,
        player_position: Position,
    ) -> (
        HashMap<String, minecraft_protocol::nbt::NbtTag>,
        Vec<&ChunkColumn>,
    ) {
        let heightmaps = HashMap::new();
        let chunk_column = Vec::new();

        return (heightmaps, chunk_column);
    }
}
