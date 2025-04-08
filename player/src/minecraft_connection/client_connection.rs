// does not include the login flow, just the "Game" section of the mc protocol.
// main function is to abstract the mc protocol for the game loop

use std::net::TcpStream;

use super::login::CommunicationError;

pub struct McClientConnection {
    socket: TcpStream,
}

impl McClientConnection {
    pub fn new(socket: TcpStream) -> Self {
        McClientConnection { socket }
    }
    
    pub fn send_tick(&self) -> Result<(), CommunicationError> {
        todo!()
    }
}
