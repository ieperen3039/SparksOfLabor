// does not include the login flow, just the "Game" section of the mc protocol.
// main function is to abstract the mc protocol for the game loop

use std::net::TcpStream;

pub struct McClientConnection {
    socket :TcpStream,
}

impl McClientConnection {

}