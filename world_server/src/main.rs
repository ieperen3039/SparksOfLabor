use sol_protocol::messages::{WorldServerReq, WorldServerRep};

use crate::network::ReplyLoop;

mod network;

extern crate zmq;
extern crate sol_voxel_engine;

fn main() {
    println!("World server: online");
    
    let handler = ReplyLoop::new("tcp://127.0.0.1:60265", handle);

    println!("World server: offline");
}

fn handle(type_value : WorldServerReq, msg : zmq::Message) -> (WorldServerRep, zmq::Message) {

}