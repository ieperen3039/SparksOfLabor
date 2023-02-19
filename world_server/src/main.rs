use sol_world_messages::{WorldServerReq, WorldServerRep};

use crate::network::ReplyLoop;

mod network;

extern crate zmq;

fn main() {
    println!("World server: online");
    
    let handler = ReplyLoop::new("tcp://127.0.0.1:60265", handle_message);

    println!("World server: offline");
}

fn handle_message(type_value : WorldServerReq, msg : zmq::Message) -> (WorldServerRep, zmq::Message) {
    
}