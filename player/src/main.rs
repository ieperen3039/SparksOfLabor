#![allow(dead_code)]

extern crate zmq;
pub mod entities;
pub mod game_event;
mod game_logic;
pub mod game_loop;
mod item_stack;
pub mod minecraft_connection;
mod player_loop;
mod player_state;
pub mod voxels;

use crate::game_loop::GameCommand;
use crate::minecraft_connection::client_connection::ClientSendCommand;
use crate::minecraft_connection::client_connection::McClientSender;
use crate::player_loop::PlayerCommand;
use crate::player_state::PlayerState;
use minecraft_connection::{
    client_connection::McClientReceiver, player_character,
    player_connect_handler::PLayerConnectHandler,
};
use sol_address_server::static_addresses;
use sol_log_server::logger_mt::LoggerMt;
use sol_voxel_lib::vector_alias::{Position, Rotation};
use std::thread;

/**
 * OK, here's what happens when a player server boots.
 *
 * We connect to the web server, which gives us an address of a log-in server. For now, we use `localhost`, later this would be an URL.
 * The web server queries the address server for the list of addresses, and sends list of names and addresses to the player server.
 * We listen in the meantime on localhost for incoming connections, until the Java client connects to us.
 * When the player tries to join, we receive player information from the client (name + UUID).
 * We connect to the "Player data server" to get the data about the player inventory, position, statistics, etc.
 * We connect to the "Player position server" to get a list of nearby players (their UUIDs)
 * We connect to the "Load balancer" to query nearby chunks and entities.
 * We send this information to the java client.
 * We start the main game loop
 */
fn main() {
    let context = zmq::Context::new();
    let logger = LoggerMt::new(
        "Player server",
        context.clone(),
        String::from(static_addresses::LOG_SERVER),
    )
    .expect("Could not connect logger");

    let registries = minecraft_vanilla::registries::get_registries();

    let (connection, client_socket) = PLayerConnectHandler::await_connect().unwrap();

    let world_server_socket = context.socket(zmq::REQ).unwrap();
    world_server_socket
        .connect(static_addresses::WORLD_SERVER)
        .unwrap();

    // TODO get world data from world_server_socket
    let mut world = voxels::world::World::new();
    let character = player_character::PlayerCharacter {
        entity_id: 0,
        uuid: [0; 4],
        position: Position::new(0.0, 60.0, 0.0),
        head_rotation: Rotation::identity(),
    };

    let player_data_socket = context.socket(zmq::REQ).unwrap();
    player_data_socket
        .connect(static_addresses::PLAYER_DATA_SERVER)
        .unwrap();

    // TODO get player data from player_data_server
    let player_state = PlayerState::new();

    // start player join
    let player_connection_data =
        PLayerConnectHandler::send_player_join(connection, &character, &mut world, client_socket)
            .expect("Could not send player join packages");

    logger.send_status("Player online");

    let (game_command_channel, game_command_receiver) = std::sync::mpsc::channel();
    let (client_comm_channel, client_comm_receiver) = std::sync::mpsc::channel();
    let (player_comm_channel, player_comm_receiver) = std::sync::mpsc::channel();

    let mut game_loop = game_loop::GameLoop::new(
        world,
        logger.clone(),
        game_command_receiver,
        client_comm_channel.clone(),
        registries.clone(),
    );
    let mut player_loop = player_loop::PlayerLoop::new(
        player_state,
        logger.clone(),
        player_comm_receiver,
        client_comm_channel.clone(),
        game_command_channel.clone(),
        registries,
    );
    let mut client_sender = McClientSender::new(
        player_connection_data.socket.try_clone().unwrap(),
        logger.clone(),
        client_comm_receiver,
    );
    let mut client_receiver = McClientReceiver::new(
        player_connection_data.socket,
        logger.clone(),
        game_command_channel.clone(),
        player_comm_channel.clone(),
    );

    let connection_send_thread = thread::spawn(move || client_sender.execute_send());
    let connection_receive_thread = thread::spawn(move || client_receiver.execute_receive());
    let player_thread = thread::spawn(move || player_loop.run());
    let game_thread = thread::spawn(move || game_loop.run());

    // when the connection is closed, the game stops
    connection_receive_thread.join().unwrap();

    // initiate stop
    game_command_channel.send(GameCommand::Stop).unwrap();
    player_comm_channel.send(PlayerCommand::Stop).unwrap();
    client_comm_channel.send(ClientSendCommand::Stop).unwrap();

    // await stop
    connection_send_thread.join().unwrap();
    game_thread.join().unwrap();
    player_thread.join().unwrap();

    logger.send_status("Player offline");
}
