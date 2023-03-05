use sol_world_messages::{WorldServerRep, WorldServerReq};

pub struct ReplyLoop {
    ctx: zmq::Context,
    endpoint: String,
    handler: fn(WorldServerReq) -> WorldServerRep,
}

pub enum ListenError {
    ZmqError(zmq::Error),
    SerialisationError(bincode::Error),
}

impl ReplyLoop {
    pub fn new(
        endpoint: &str,
        handler: fn(WorldServerReq) -> WorldServerRep,
    ) -> ReplyLoop {
        ReplyLoop {
            ctx: zmq::Context::new(),
            endpoint: String::from(endpoint),
            handler,
        }
    }

    fn start_listen(&self) -> Result<(), ListenError> {
        let socket = self
            .ctx
            .socket(zmq::REP)
            .map_err(|err| ListenError::ZmqError(err))?;
        socket
            .bind(self.endpoint.as_str())
            .map_err(|err| ListenError::ZmqError(err))?;

        let mut i = 0;
        let mut message_type_bytes: [u8; 1] = [0];
        let receive_flags = 0;
        let send_flags = 0;

        loop {
            // TODO handle zmq errors here
            let encoded = socket
                .recv_bytes(receive_flags)
                .map_err(|err| ListenError::ZmqError(err))?;

            let request = bincode::deserialize(&encoded[..])
                .map_err(|err| ListenError::SerialisationError(err))?;

            let reply = (self.handler)(request);

            let encoded = bincode::serialize(&reply) //
                .map_err(|err| ListenError::SerialisationError(err))?;

            socket
                .send(encoded, send_flags)
                .map_err(|err| ListenError::ZmqError(err))?;
        }
    }
}
