
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum NetworkError {
    ZmqError(zmq::Error),
    SerialisationError(bincode::Error),
}

pub struct ReplyLoop<Req, Rep> {
    socket: zmq::Socket,
    endpoint: String,
    handler: fn(Req) -> Rep,
}

impl<Req, Rep> ReplyLoop<Req, Rep>
where
    for<'a> Req: Serialize + Deserialize<'a>,
    for<'a> Rep: Serialize + Deserialize<'a>,
{
    pub fn new(
        context: zmq::Context,
        endpoint: String,
        handler: fn(Req) -> Rep,
    ) -> Result<ReplyLoop<Req, Rep>, zmq::Error> {
        let socket = context.socket(zmq::REP)?;
        Ok(ReplyLoop {
            socket,
            endpoint,
            handler,
        })
    }

    pub fn listen_until_stop(&self) -> Result<(), NetworkError> {
        self.socket
            .bind(self.endpoint.as_str())
            .map_err(|err| NetworkError::ZmqError(err))?;

        let receive_flags = 0x00;
        let send_flags = zmq::DONTWAIT;

        loop {
            // TODO handle zmq errors here
            let request = await_receive(&self.socket, receive_flags)?;

            let reply: Rep = (self.handler)(request);

            send(&self.socket, reply, send_flags)?;
        }
    }
}

pub fn query<Req, Rep>(
    socket: &zmq::Socket,
    request: Req,
) -> Result<Rep, NetworkError>
where
    for<'a> Req: Serialize + Deserialize<'a>,
    for<'a> Rep: Serialize + Deserialize<'a>,
{
    assert!(socket.get_socket_type().is_ok());
    assert_ne!(socket.get_socket_type(), Ok(zmq::PUB));

    send(socket, request, 0x00)?;
    await_receive(socket, 0x00)
}

pub fn send<T>(
    socket: &zmq::Socket,
    object: T,
    send_flags: i32,
) -> Result<(), NetworkError>
where
    for<'a> T: Serialize + Deserialize<'a>,
{
    let encoded = bincode::serialize(&object) //
        .map_err(|err| NetworkError::SerialisationError(err))?;

    socket
        .send(encoded, send_flags)
        .map_err(|err| NetworkError::ZmqError(err))?;

    Ok(())
}

pub fn await_receive<T>(
    socket: &zmq::Socket,
    receive_flags: i32,
) -> Result<T, NetworkError>
where
    for<'a> T: Serialize + Deserialize<'a>,
{
    let encoded = socket
        .recv_bytes(receive_flags)
        .map_err(|err| NetworkError::ZmqError(err))?;

    let object: T =
        bincode::deserialize(&encoded[..]).map_err(|err| NetworkError::SerialisationError(err))?;

    Ok(object)
}
