use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

use minecraft_protocol::packets as mc_packets;
use sol_voxel_lib::world::World;

use super::network;
use super::{
    login::{self, CommunicationError, PlayerConnectionData},
    player_character::PlayerCharacter,
};

pub struct Connection {}

pub struct Socket<'world> {
    world: &'world World,
}

impl Socket<'_> {
    pub fn await_connect(&self) -> Result<(PlayerConnectionData, TcpStream), CommunicationError> {
        let listener = TcpListener::bind("127.0.0.1:25567").expect("Failed to listen");

        // Accept 1 incoming connections
        let (mut stream, _addr) = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_millis(50)))?;

        let mut buffer = Vec::new();
        let handshake_packet: mc_packets::handshake::ServerboundPacket =
            network::receive_packet(&mut stream, &mut buffer)?;

        let mc_packets::handshake::ServerboundPacket::Hello {
            protocol_version: _,
            server_address: _,
            server_port: _,
            next_state,
        } = handshake_packet;

        match next_state {
            mc_packets::ConnectionState::Login => {
                let player_info = login::login(&mut stream)?;

                // TODO keep the player busy while we connect to the back-end
                return Ok((player_info, stream));
            },
            mc_packets::ConnectionState::Status => {
                todo!("Handle ConnectionState::Status")
            },
            _ => {
                return Err(CommunicationError::UnexpectedState {
                    from: mc_packets::ConnectionState::HandShake,
                    to: next_state,
                })
            },
        };
    }

    pub fn join_game(
        &self,
        player: PlayerConnectionData,
        character: &PlayerCharacter,
        socket: TcpStream,
    ) -> Result<(), CommunicationError> {
        // player is spawning

        let player_info = login::initialize_client(socket, player, character)?;
        login::send_initial_chunk_data(&mut player_info.socket, self.world, character.positon)?;
        let eid = self.world.spawn_player(player_info, character.positon);

        return Ok(());
    }
}
