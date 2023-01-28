extern crate zmq;

// let 60265 be the SOL port

fn main() {
    println!("World server: online");
    let ctx = zmq::Context::new();

    let socket = ctx.socket(zmq::REP).unwrap();
    socket.bind("tcp://127.0.0.1:60265").unwrap();

    let mut i = 0;

    while i < 5 {
        let msg_string = socket.recv_string(0).unwrap().unwrap();
        println!("Received {msg_string}");

        let reply = std::format!("Received {i}th message");
        i += 1;
        socket.send(reply.as_str(), 0).unwrap();
    }

    println!("World server: offline");
}