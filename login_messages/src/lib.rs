pub mod messages {
    use serde::{Deserialize, Serialize};

    pub const VERSION_STRING: &str = "0.1"; // this could obviously be better

    #[derive(Serialize, Deserialize)]
    pub enum General {
        Ping,
        Pong,
        ProtocolConnect {
            version: String,
            connection_name: String,
        },
    }
}
