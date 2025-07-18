// does not include the login flow, just the "Game" section of the mc protocol.
// main function is to abstract the mc protocol for the game loop

use super::login::CommunicationError;
use crate::game_loop::GameCommand;
use crate::minecraft_connection::network;
use crate::player_handler::{PlaceBlockCommand, PlayerCommand};
use minecraft_protocol::packets::play_clientbound::ClientboundPacket;
use minecraft_protocol::packets::play_serverbound::ServerboundPacket;
use minecraft_protocol::MinecraftPacketPart;
use sol_log_server::logger_mt::LoggerMt;
use sol_voxel_lib::vector_alias::Coordinate;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;

pub struct McClientReceiver {
    socket: TcpStream,
    logger: LoggerMt,
    world_event_channel: mpsc::Sender<GameCommand>,
    player_event_channel: mpsc::Sender<PlayerCommand>,
}

pub enum ClientSendCommand {
    Stop,
    Message(Vec<u8>),
}

impl ClientSendCommand {
    pub fn try_from<'a, Packet: MinecraftPacketPart<'a>>(
        msg: Packet,
    ) -> Result<Self, CommunicationError> {
        msg.serialize_minecraft_packet()
            .map(|serialized| ClientSendCommand::Message(serialized))
            .map_err(|e| CommunicationError::SerializationError(e.to_string()))
    }
}

pub struct McClientSender {
    socket: TcpStream,
    logger: LoggerMt,
    client_comm_queue: mpsc::Receiver<ClientSendCommand>,
}

impl McClientReceiver {
    pub fn new(
        socket: TcpStream,
        logger: LoggerMt,
        world_event_channel: mpsc::Sender<GameCommand>,
        player_event_channel: mpsc::Sender<PlayerCommand>,
    ) -> Self {
        McClientReceiver {
            socket,
            logger,
            world_event_channel,
            player_event_channel,
        }
    }

    pub fn execute_receive(&mut self) {
        loop {
            let mut buffer = Vec::new();

            let packet = network::receive_packet(&mut self.socket, &mut buffer);

            let packet = match packet {
                Ok(p) => p,
                Err(CommunicationError::ConnectionClosed) => return,
                Err(err) => {
                    println!("Error while receiving message: {err:?}");
                    continue;
                },
            };

            // also avoids lifetime issues
            let packet_name = format!("{:?}", packet);

            println!("Received {packet_name}");

            let result = match packet {
                ServerboundPacket::RequestPing { payload } => network::send_packet(
                    &mut self.socket,
                    ClientboundPacket::PingResponse { payload },
                )
                    .map_err(|e| CommunicationError::IoError(e)),
                ServerboundPacket::PlaceBlock {
                    hand,
                    location,
                    face,
                    cursor_position_x,
                    cursor_position_y,
                    cursor_position_z,
                    inside_block,
                    sequence,
                } => self
                    .player_event_channel
                    .send(PlayerCommand::PlaceBlock(PlaceBlockCommand {
                        hand,
                        location: Coordinate::new(location.x, location.y as i32, location.z),
                        face,
                        cursor_position_x,
                        cursor_position_y,
                        cursor_position_z,
                        inside_block,
                    }))
                    .map_err(|e| CommunicationError::InternalError(format!("{e:?}"))),
                ServerboundPacket::UseItem { .. } => Ok(()),

                // may be ignored
                ServerboundPacket::KeepAlive { .. } => Ok(()),
                ServerboundPacket::Pong { .. } => Ok(()),

                // TODO
                ServerboundPacket::ConfirmTeleportation { .. } => Ok(()),
                ServerboundPacket::QueryBlockNbt { .. } => Ok(()),
                ServerboundPacket::ChangeDifficulty { .. } => Ok(()),
                ServerboundPacket::AcknowledgeMessage { .. } => Ok(()),
                ServerboundPacket::ChatCommand { .. } => Ok(()),
                ServerboundPacket::ChatMessage { .. } => Ok(()),
                ServerboundPacket::PlayerSession { .. } => Ok(()),
                ServerboundPacket::ChunkBatchReceived { .. } => Ok(()),
                ServerboundPacket::ClientStatus { .. } => Ok(()),
                ServerboundPacket::ClientSettings { .. } => Ok(()),
                ServerboundPacket::CommandSuggestionsRequest { .. } => Ok(()),
                ServerboundPacket::AcknowledgeConfiguration => Ok(()),
                ServerboundPacket::ClickWindowButton { .. } => Ok(()),
                ServerboundPacket::ClickWindowSlot { .. } => Ok(()),
                ServerboundPacket::CloseWindow { .. } => Ok(()),
                ServerboundPacket::PluginMessage { .. } => Ok(()),
                ServerboundPacket::EditBook { .. } => Ok(()),
                ServerboundPacket::QueryEntityNbt { .. } => Ok(()),
                ServerboundPacket::InteractEntity { .. } => Ok(()),
                ServerboundPacket::GenerateStructure { .. } => Ok(()),
                ServerboundPacket::LockDifficulty { .. } => Ok(()),
                ServerboundPacket::SetPlayerPosition { .. } => Ok(()),
                ServerboundPacket::SetPlayerPositionAndRotation { .. } => Ok(()),
                ServerboundPacket::SetPlayerRotation { .. } => Ok(()),
                ServerboundPacket::SetPlayerOnGround { .. } => Ok(()),
                ServerboundPacket::MoveVehicle { .. } => Ok(()),
                ServerboundPacket::PaddleBoat { .. } => Ok(()),
                ServerboundPacket::PickItem { .. } => Ok(()),
                ServerboundPacket::PlaceRecipe { .. } => Ok(()),
                ServerboundPacket::PlayerAbilities { .. } => Ok(()),
                ServerboundPacket::DigBlock { .. } => Ok(()),
                ServerboundPacket::PlayerAction { .. } => Ok(()),
                ServerboundPacket::SteerVehicle { .. } => Ok(()),
                ServerboundPacket::ChangeRecipeBookSettings { .. } => Ok(()),
                ServerboundPacket::SetSeenRecipe { .. } => Ok(()),
                ServerboundPacket::RenameItem { .. } => Ok(()),
                ServerboundPacket::ResourcePackStatus { .. } => Ok(()),
                ServerboundPacket::SetSeenAdvancements { .. } => Ok(()),
                ServerboundPacket::SelectTrade { .. } => Ok(()),
                ServerboundPacket::SetBeaconEffect { .. } => Ok(()),
                ServerboundPacket::SetHeldItem { .. } => Ok(()),
                ServerboundPacket::ProgramCommandBlock { .. } => Ok(()),
                ServerboundPacket::ProgramCommandBlockMinecart { .. } => Ok(()),
                ServerboundPacket::SetCreativeModeSlot { .. } => Ok(()),
                ServerboundPacket::ProgramJigsawBlock { .. } => Ok(()),
                ServerboundPacket::ProgramStrutureBlock { .. } => Ok(()),
                ServerboundPacket::UpdateSign { .. } => Ok(()),
                ServerboundPacket::SwingArms { .. } => Ok(()),
                ServerboundPacket::Spectate { .. } => Ok(()),
            };

            if result.is_err() {
                println!("Error while handling message {packet_name}: {result:?}")
            }
        }
    }
}

impl McClientSender {
    pub fn new(
        socket: TcpStream,
        logger: LoggerMt,
        client_comm_queue: mpsc::Receiver<ClientSendCommand>,
    ) -> Self {
        McClientSender {
            socket,
            logger,
            client_comm_queue,
        }
    }

    pub fn execute_send(&mut self) {
        loop {
            let command = self.client_comm_queue.recv().unwrap();
            match command {
                ClientSendCommand::Stop => return,
                ClientSendCommand::Message(msg) => {
                    let result = self.socket.write_all(&msg);

                    if result.is_err() {
                        println!("Error while sending message: {result:?}");
                    }
                },
            }
        }
    }
}
