// does not include the login flow, just the "Game" section of the mc protocol.
// main function is to abstract the mc protocol for the game loop

use super::login::CommunicationError;
use crate::game_event::GameEvent;
use crate::game_loop::GameCommand;
use crate::minecraft_connection::network;
use minecraft_protocol::packets::play_clientbound::ClientboundPacket;
use minecraft_protocol::packets::play_serverbound::ServerboundPacket;
use std::io::Read;
use std::net::TcpStream;
use std::sync::mpsc;

pub struct McClientConnection {
    socket: TcpStream,
    world_event_channel: mpsc::Sender<GameCommand>,
}

impl McClientConnection {
    pub fn new(socket: TcpStream, world_event_channel: mpsc::Sender<GameCommand>) -> Self {
        McClientConnection {
            socket,
            world_event_channel,
        }
    }

    pub fn run(&mut self) {
        loop {
            let mut buffer = Vec::with_capacity(1);

            let packet =
                network::receive_packet(&mut self.socket, &mut buffer);

            let packet = match packet {
                Ok(p) => p,
                Err(CommunicationError::ConnectionClosed) => return,
                Err(err) => {
                    println!("Error while receiving message: {err:?}");
                    continue;
                }
            };

            println!("Received {packet:?}");

            let result = match packet {
                ServerboundPacket::RequestPing { payload } => self.send_ping(payload as i64),
                ServerboundPacket::PlaceBlock { .. } => Ok(()),
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
                println!("Error while handling message {packet:?}: {result:?}")
            }
        }
    }

    fn send_ping(&mut self, payload: i64) -> Result<(), CommunicationError> {
        network::send_packet(
            &mut self.socket,
            ClientboundPacket::PingResponse { payload },
        )?;
        Ok(())
    }
}
