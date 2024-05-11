#![allow(dead_code)]

pub mod minecraft_connection;

use sol_address_server::static_addresses;
use sol_log_server::log::Logger;
use sol_network_lib::network::{self, NetworkError};
use sol_world_messages::{WorldServerRep, WorldServerReq};

extern crate zmq;
/**
 * OK, here's what happes when a player boots.
 * 
 * First we connect to the web server, which gives us an address of a log-in server. For now we use `localhost`, later this would be an URL.
 * The web server queries the address server for the list of addresses, and sends list of names and addresses to the player.
 * The player connects to the "Player data server" to get the data about the player inventory, position, statistics, UUID etc.
 * The player connects to the "Player position server" to get a list of nearby players (their UUIDs)
 * The player connects to the "Load balancer" to query nearby chunks and entities.
 * 
 */
fn main() {
    let context = zmq::Context::new();
    let logger = Logger::new(
        "Player",
        context.clone(),
        String::from(static_addresses::LOG_SERVER),
    )
    .expect("Could not connect logger");

    let socket = context.socket(zmq::REQ).unwrap();
    socket.connect(static_addresses::WORLD_SERVER).unwrap();

    logger.send_status("Player online");

    for i in 0..3 {
        let request = WorldServerReq::Ping(std::format!("Sending {i}th message"));
        let result: Result<WorldServerRep, NetworkError> = network::query(&socket, request);

        match result {
            Ok(WorldServerRep::Pong(text)) => logger.send_debug(&format!("received pong ({text})")),
            Ok(_) => logger.send_debug("received not-pong"),
            Err(err) => println!("Error receiving message {:?}", err),
        }

        std::thread::sleep(core::time::Duration::from_millis(200));
    }

    logger.send_status("Player offline");
}
