use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

use minecraft_protocol::packets as mc_packets;

use super::login::{self, CommunicationError};
use super::network;

pub struct Connection {}

impl Connection {
    pub fn await_connect() -> Result<Connection, CommunicationError> {
        let listener = TcpListener::bind("127.0.0.1:25567").expect("Failed to listen");

        // Accept 1 incoming connections
        let (stream, addr) = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_millis(50)))?;

        let mut buffer = Vec::new();
        let handshake_packet: mc_packets::handshake::ServerboundPacket =
            network::receive_packet(&mut stream, &mut buffer)?;

        let mc_packets::handshake::ServerboundPacket::Hello {
            protocol_version,
            server_address,
            server_port,
            next_state,
        } = handshake_packet
        else {
            unreachable!()
        };

        let player_info = match next_state {
            mc_packets::ConnectionState::Login => {
                login::login(&mut stream, addr)?
            },
            mc_packets::ConnectionState::Status => {
                todo!("Handle ConnectionState::Status")
            },
            _ => {
                return Err(CommunicationError::UnexpectedState {
                    from: mc_packets::ConnectionState::HandShake,
                    to: next_state
                })
            },
        };

        let player_info = login::initialize_client(&mut stream, player_info, world)?;
        let uuid = player_info.uuid;
        let eid = world.spawn_player(stream, player_info);

        return Ok(Connection {});
    }
}
