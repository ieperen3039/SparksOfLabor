use serde::{Deserialize, Serialize};
use sol_voxel_lib::{
    chunk::{Chunk16, Chunk4, Chunk64},
    vector_alias::Coordinate,
};

pub const VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

pub const CONNECTION_NAME_WORLD_SERVER_REQ: &str = "WorldServerRequest";

#[derive(Serialize, Deserialize)]
pub enum WorldServerReq {
    ContentChunk4(Coordinate),
    ContentChunk64(Coordinate),
    Ping(String),
}

pub const CONNECTION_NAME_WORLD_SERVER_REP: &str = "WorldServerReply";

#[derive(Serialize, Deserialize)]
pub enum WorldServerRep {
    Pong,
    ContentChunk4Denied,
    ContentChunk4(Coordinate, Box<Chunk4>),
    ContentChunk16Denied,
    ContentChunk16(Coordinate, Box<Chunk16>),
    ContentChunk64Denied,
    ContentChunk64(Coordinate, Box<Chunk64>),
}
