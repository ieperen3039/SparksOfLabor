
use serde::{Serialize, Deserialize};
use sol_voxel_lib::{vector_alias::Coordinate, chunk::Chunk64};

pub const VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

pub const CONNECTION_NAME_WORLD_SERVER_REQ : &str = "WorldServerRequest";

#[derive(Serialize, Deserialize)]
pub enum WorldServerReq {
    ContentChunk4(Coordinate)
}

pub const CONNECTION_NAME_WORLD_SERVER_REP : &str = "WorldServerReply";

#[derive(Serialize, Deserialize)]
pub enum WorldServerRep {
    Non,
    ContentChunk64(Coordinate, Chunk64)
}