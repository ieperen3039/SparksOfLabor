#![allow(dead_code)]

pub mod minecraft_connection;
pub mod game_loop;
pub mod voxels;
pub mod entities;
pub mod game_event;

use minecraft_connection::{client_connection::McClientConnection, minecraft_socket::Connection, player_character};
use sol_address_server::static_addresses;
use sol_log_server::log::Logger;
use sol_network_lib::network::{self, NetworkError};
use sol_voxel_lib::vector_alias::{Position, Rotation};
use sol_world_messages::{WorldServerRep, WorldServerReq};

extern crate zmq;
/**
 * OK, here's what happes when a player server boots.
 * 
 * We connect to the web server, which gives us an address of a log-in server. For now we use `localhost`, later this would be an URL.
 * The web server queries the address server for the list of addresses, and sends list of names and addresses to the player server.
 * We listen in the mean time on localhost for incoming connections, until the Java client connects to us.
 * When the player tries to join, we receive player information from the client (name + UUID).
 * We connect to the "Player data server" to get the data about the player inventory, position, statistics etc.
 * We connect to the "Player position server" to get a list of nearby players (their UUIDs)
 * We connect to the "Load balancer" to query nearby chunks and entities.
 * We send this information to the java client.
 * We start the main game loop
 */
fn main() {
    let context = zmq::Context::new();
    let logger = Logger::new(
        "Player server",
        context.clone(),
        String::from(static_addresses::LOG_SERVER),
    )
    .expect("Could not connect logger");

    let (connection, client_socket) = Connection::await_connect().unwrap();

    let world_server_socket = context.socket(zmq::REQ).unwrap();
    world_server_socket.connect(static_addresses::WORLD_SERVER).unwrap();

    // get world data from world_server_socket
    let mut world = voxels::world::World::new();
    let character = player_character::PlayerCharacter{ entity_id: 0, uuid: [0; 4], positon: Position::new(0.0, 60.0, 0.0), head_rotation: Rotation::identity() };

    // start player join
    let player_connection_data = Connection::send_player_join(connection, &character, &mut world, client_socket)
        .expect("Could not send player join packages");

    logger.send_status("Player online");

    let mut game_state = game_loop::GameState::build(world);
    game_state.run(McClientConnection::new(player_connection_data.socket));

    logger.send_status("Player offline");
}
