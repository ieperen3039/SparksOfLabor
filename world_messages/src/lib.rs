use serde::{Deserialize, Serialize};
use sol_voxel_lib::{
    chunk::{Chunk16, Chunk64}, vector_alias::Coordinate, voxel::Voxel
};

pub const VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

pub const CONNECTION_NAME_WORLD_SERVER_REQ: &str = "WorldServerRequest";

#[derive(Serialize, Deserialize)]
pub enum WorldServerReq {
    Ping(String),
    ContentChunk16(Coordinate),
    ContentChunk64(Coordinate),
    SetVoxel(Coordinate, Voxel),
}

pub const CONNECTION_NAME_WORLD_SERVER_REP: &str = "WorldServerReply";

#[derive(Serialize, Deserialize)]
pub enum WorldServerRep {
    Pong(String),
    RequestDenied(WorldServerReq),
    ContentChunk16(Coordinate, Box<Chunk16>),
    ContentChunk64(Coordinate, Box<Chunk64>),
}
