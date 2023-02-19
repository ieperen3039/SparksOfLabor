pub mod messages {
    use serde::{Serialize, Deserialize};
    use sol_voxel_engine::vector_alias::Coordinate;

    pub const VERSION_STRING : &str = "0.1"; // this could obviously be better

    #[derive(Serialize, Deserialize)]
    pub enum General {
        Ping, 
        Pong,
        ProtocolConnect { version : String, connection_name : String},
    }

    pub const CONNECTION_NAME_WORLD_SERVER_REQ : &str = "WorldServerRequest";

    #[derive(Serialize, Deserialize)]
    pub enum WorldServerReq {
        ContentChunk4(Coordinate)
    }

    pub const CONNECTION_NAME_WORLD_SERVER_REP : &str = "WorldServerReply";
    
    #[derive(Serialize, Deserialize)]
    pub enum WorldServerRep {
        ContentChunk4(Coordinate, Chunk4)
    }
}
