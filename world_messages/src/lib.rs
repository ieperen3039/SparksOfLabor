use serde::{Deserialize, Serialize};
use sol_voxel_lib::{
    vector_alias::Coordinate,
};

pub const VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

pub const CONNECTION_NAME_WORLD_SERVER_REQ: &str = "WorldServerRequest";

#[derive(Serialize, Deserialize)]
pub enum WorldServerReq {
    Ping(String),
    ContentChunk4(Coordinate),
    ContentChunk16(Coordinate),
    ContentChunk64(Coordinate),
    SetVoxel(Coordinate, Voxel),
    SetChunk4(Coordinate, Chunk4),
}

pub const CONNECTION_NAME_WORLD_SERVER_REP: &str = "WorldServerReply";

#[derive(Serialize, Deserialize)]
pub enum WorldServerRep {
    Pong(String),
    RequestDenied(WorldServerReq),
    ContentChunk4(Coordinate, Box<Chunk4>),
    ContentChunk16(Coordinate, Box<Chunk16>),
    ContentChunk64(Coordinate, Box<Chunk64>),
}
