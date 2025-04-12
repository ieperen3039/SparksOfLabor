use std::{
    net::{TcpListener, TcpStream},
    time::Duration,
};

use crate::voxels::world::World;
use minecraft_protocol::packets as mc_packets;
use sol_address_server::static_addresses;

use super::network;
use super::{
    login::{self, CommunicationError, PlayerConnectionData},
    player_character::PlayerCharacter,
};

const CLIENT_CONNECTION_TIMEOUT: Duration = Duration::from_millis(500);

pub struct PLayerConnectHandler {}

impl PLayerConnectHandler {
    pub fn await_connect() -> Result<(PlayerConnectionData, TcpStream), CommunicationError> {
        loop {
            let listener = TcpListener::bind(static_addresses::MINECRAFT_SERVER_BIND)
                .expect("Failed to listen");

            // Accept 1 incoming connections
            let (mut stream, _addr) = listener.accept()?;
            stream.set_read_timeout(Some(CLIENT_CONNECTION_TIMEOUT))?;

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
                mc_packets::ConnectionState::Status => {
                    Self::handle_status_state(&mut stream)?;
                    // continue waiting for connections
                },
                mc_packets::ConnectionState::Login => {
                    let player_info = login::login(&mut stream)?;

                    // TODO keep the player busy while we connect to the back-end
                    return Ok((player_info, stream));
                },
                _ => {
                    return Err(CommunicationError::UnexpectedState {
                        from: mc_packets::ConnectionState::HandShake,
                        to: next_state,
                    })
                },
            };
        }
    }

    fn handle_status_state(stream: &mut TcpStream) -> Result<(), CommunicationError> {
        let mut buffer = Vec::new();
        loop {
            let packet: mc_packets::status::ServerboundPacket =
                network::receive_packet(stream, &mut buffer)?;

            match packet {
                mc_packets::status::ServerboundPacket::Request => {
                    login::send_status_response(stream)?;
                    // continue handling packets
                },
                mc_packets::status::ServerboundPacket::Ping { payload } => {
                    login::pong(stream, payload)?;
                    return Ok(());
                },
            }
        }
    }

    pub fn send_player_join(
        player: PlayerConnectionData,
        character: &PlayerCharacter,
        world: &mut World,
        socket: TcpStream,
    ) -> Result<login::PlayerInfo, CommunicationError> {
        // player is spawning

        let mut player_info = login::initialize_client(socket, player, character)?;
        let chunks_per_tick =
            login::send_initial_chunk_data(&mut player_info.socket, world, character.position)?;
        player_info.chunks_per_tick = chunks_per_tick;

        Ok(player_info)
    }
}
