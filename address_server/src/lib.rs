pub mod static_addresses {
    // not a zmq address
    pub const MINECRAFT_SERVER_BIND : &str = "127.0.0.1:25567";
    // zmq addresses
    pub const LOG_SERVER : &str = "ipc://tmp/log_server";
    pub const WORLD_SERVER : &str = "ipc://tmp/world_server";
}