use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
    io
};

use super::login;

pub struct Connection {}

impl Connection {
    pub fn await_connect() -> io::Result<Connection> {
        let listener = TcpListener::bind("127.0.0.1:25567").expect("Failed to listen");

        // Accept 1 incoming connections
        let (stream, addr) = listener.accept()?;
        
        login::handle_connection(stream, addr)?;
        
        let player_info = login::initialize_client(&mut stream, player_info, world)?;
        let uuid = player_info.uuid;
        let eid = world.spawn_player(world, stream, player_info);

        return Ok(Connection{});
    }

}
