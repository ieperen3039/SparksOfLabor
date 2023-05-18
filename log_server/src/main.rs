use sol_log_server::{LogText, LoggingMessage};
use time::OffsetDateTime;

pub enum ListenError {
    ZmqError(zmq::Error),
    SerialisationError(bincode::Error),
}

fn main() {
    let context = zmq::Context::new();

    let socket = context.socket(zmq::SUB).expect("Could not create socket");

    loop {
        let result = listen(&socket, 0x00);

        if let Err(error) = result {
            let current_time = OffsetDateTime::now_utc();
            match error {
                ListenError::ZmqError(error) => handle(LoggingMessage::Text(LogText {
                    timestamp: current_time,
                    level: sol_log_server::LogType::RecoverableError,
                    text: format!("ZeroMQ error: {error}"),
                })),
                ListenError::SerialisationError(error) => handle(LoggingMessage::Text(LogText {
                    timestamp: current_time,
                    level: sol_log_server::LogType::RecoverableError,
                    text: format!("Serialisation error: {error}"),
                })),
            }
        }
    }
}

fn listen(
    socket: &zmq::Socket,
    receive_flags: i32,
) -> Result<(), ListenError> {

    // TODO handle zmq errors here
    let encoded = socket
        .recv_bytes(receive_flags)
        .map_err(|err| ListenError::ZmqError(err))?;

    let message =
        bincode::deserialize(&encoded[..]).map_err(|err| ListenError::SerialisationError(err))?;

    handle(message);

    Ok(())
}

fn handle(message: LoggingMessage) {}
