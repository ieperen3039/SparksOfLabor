use sol_world_messages::{WorldServerReq, WorldServerRep};

use crate::{network::ReplyLoop, world::World};

mod network;
pub mod world;

extern crate zmq;

fn main() {
    let world = World::new();

    println!("World server: online");
    
    let handler = ReplyLoop::new("tcp://127.0.0.1:60265", handle_message);

    println!("World server: offline");
}

fn handle_message(type_value : WorldServerReq) -> WorldServerRep {
    WorldServerRep::Non
}