#![allow(dead_code)]

use sol_address_server::static_addresses;
use sol_log_server::log::Logger;
use sol_network_lib::network::{self, NetworkError};
use sol_world_messages::{WorldServerRep, WorldServerReq};

extern crate zmq;

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
