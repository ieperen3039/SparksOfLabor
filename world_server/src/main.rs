#![allow(dead_code)]

use sol_address_server::static_addresses;
use sol_log_server::log::{Logger, Severity};
use sol_network_lib::network::{NetworkError, ReplyLoop};
use sol_world_messages::{WorldServerRep, WorldServerReq};

use crate::world::World;
mod world;

extern crate zmq;

fn main() {
    let context = zmq::Context::new();
    let logger = Logger::new(
        "World Server",
        context.clone(),
        String::from(static_addresses::LOG_SERVER),
    )
    .expect("Could not connect logger");

    let reply_loop = {
        let reply_loop_result = ReplyLoop::new(
            context.clone(),
            String::from(static_addresses::WORLD_SERVER),
            handle_message,
        );

        match reply_loop_result {
            Ok(reply_loop) => reply_loop,
            Err(error) => {
                logger.log(
                    Severity::FatalError,
                    &format!("Could not create reply loop: {error}"),
                );
                return;
            },
        }
    };

    let world = World::new();

    logger.send_status("World server online");

    let stop_reason = reply_loop.start_listen();

    logger.send_status("World server offline");

    match stop_reason {
        Ok(_) => {},
        Err(NetworkError::ZmqError(error)) => {
            logger.log(Severity::FatalError, &format!("ZeroMQ error: {error}"))
        },
        Err(NetworkError::SerialisationError(error)) => logger.log(
            Severity::FatalError,
            &format!("Serialisation error: {error}"),
        ),
    }
}

fn handle_message(message: WorldServerReq) -> WorldServerRep {
    println!("Received something");
    match message {
        WorldServerReq::Ping(msg) => WorldServerRep::Pong(msg),
        _ => WorldServerRep::RequestDenied(message),
    }
}
