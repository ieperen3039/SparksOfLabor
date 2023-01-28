#![allow(dead_code)]
// #extern crate glfw;
// pub mod vector_alias;
// pub mod voxel_engine;

extern crate zmq;

fn main() {
    println!("Player: online");
    let ctx = zmq::Context::new();

    let socket = ctx.socket(zmq::REQ).unwrap();
    socket.connect("tcp://127.0.0.1:60265").unwrap();

    for i in 0..3 {
        let request = std::format!("Sending {i}th message");
        socket.send(request.as_str(), 0).unwrap();

        let msg_string = socket.recv_string(0).unwrap().unwrap();
        println!("Received {msg_string}");
    }

    println!("Player: offline");
}
