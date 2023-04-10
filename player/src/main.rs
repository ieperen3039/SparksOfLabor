#![allow(dead_code)]

use rendering::render::RenderEngine;

mod rendering;

extern crate zmq;

fn main() {
    // println!("Player: online");
    // let ctx = zmq::Context::new();

    // let socket = ctx.socket(zmq::REQ).unwrap();
    // socket.connect("tcp://127.0.0.1:60265").unwrap();

    // for i in 0..3 {
    //     let request = std::format!("Sending {i}th message");
    //     socket.send(request.as_str(), 0).unwrap();

    //     let msg_string = socket.recv_string(0).unwrap().unwrap();
    //     println!("Received {msg_string}");
    // }

    // println!("Player: offline");
    let render_engine = RenderEngine::new(800, 800)
        .expect("Could not create render engine");
    
    render_engine.run_until_close();
}
