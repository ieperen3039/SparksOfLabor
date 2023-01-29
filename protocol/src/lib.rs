pub mod messages {
    use serde::{Serialize, Deserialize};
    use sol_voxel_engine::vector_alias::Coordinate;

    pub const VERSION : &str = "0.1"; // this could obviously be better

    #[derive(Serialize, Deserialize)]
    pub enum General {
        Ping, 
        Pong,
        ProtocolConnect(String /* = VERSION */, String /* Entity to connect to */),
    }

    pub const ENTITY_WORLD_SERVER_REQ : &str = "WorldServerReq";
    pub enum WorldServerReq {
        ContentChunk2(Coordinate)
    }

    pub const ENTITY_WORLD_SERVER_REP : &str = "WorldServerRep";
    pub enum WorldServerRep {
        ContentChunk2(Coordinate)
    }
}
