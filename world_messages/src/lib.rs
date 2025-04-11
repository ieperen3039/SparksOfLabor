use serde::{Deserialize, Serialize};
use sol_voxel_lib::{
    chunk16::{Chunk16}, vector_alias::Coordinate, voxel::Voxel
};
use sol_voxel_lib::chunk_column::ChunkColumn;
use sol_voxel_lib::vector_alias::{ChunkColumnCoordinate, Coordinate16};

pub const VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

pub const CONNECTION_NAME_WORLD_SERVER_REQ: &str = "WorldServerRequest";

#[derive(Serialize, Deserialize)]
pub enum WorldServerReq {
    Ping(String),
    ContentChunk16(Coordinate16),
    ContentChunkColumn(ChunkColumnCoordinate),
    SetVoxel(Coordinate, Voxel),
}

pub const CONNECTION_NAME_WORLD_SERVER_REP: &str = "WorldServerReply";

#[derive(Serialize, Deserialize)]
pub enum WorldServerRep {
    Pong(String),
    ContentChunk16(Coordinate16, Box<Chunk16>),
    ContentChunkColumn(ChunkColumnCoordinate, Box<ChunkColumn>),
    SetVoxelAcknowledged(Coordinate),
    SetVoxelDenied(Coordinate),
    Empty,
}
