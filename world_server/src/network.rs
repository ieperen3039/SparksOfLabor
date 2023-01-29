use sol_protocol::messages::{WorldServerReq, WorldServerRep};

pub struct ReplyLoop {
    ctx : zmq::Context,
    endpoint : String,
    handler : fn(WorldServerReq, zmq::Message) -> (WorldServerRep, zmq::Message),
}

impl ReplyLoop {
    pub fn new(endpoint : &str, handler : fn(WorldServerReq, zmq::Message) -> (WorldServerRep, zmq::Message)) -> ReplyLoop {
        ReplyLoop {
            ctx : zmq::Context::new(),
            endpoint : String::from(endpoint),
            handler,
        }
    }

    fn start_listen(&self) -> Result<(), zmq::Error> {
        let socket = self.ctx.socket(zmq::REP)?;
        socket.bind(self.endpoint.as_str())?;

        let mut i = 0;
        let mut message_type_bytes : [u8; 1] = [0];
        let receive_flags = 0;
        let send_flags = 0;

        loop {
            // TODO handle zmq errors here
            let message_length = socket.recv_into(&mut message_type_bytes, receive_flags)?;
            if message_length != 1 { continue; }
            let message_type = message_type_bytes[0];

            let message_body = socket.recv_msg(receive_flags)?;

            let (response_id, response_message) = (self.handler)(message_type, message_body);

            message_type_bytes[0] = response_id;
            socket.send(&message_type_bytes[..], send_flags)?;
            socket.send(response_message, send_flags)?;
        }
    }
}