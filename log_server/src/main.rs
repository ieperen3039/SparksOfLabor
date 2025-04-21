use sol_address_server::static_addresses;
use sol_log_server::*;
use sol_network_lib::network::NetworkError;
use time::OffsetDateTime;

const LOG_SERVER_NAME: &str = "Log server";

fn main() {
    let context = zmq::Context::new();

    let socket = {
        let socket_result = create_listen_socket(context);
        match socket_result {
            Err(err) => {
                println!("Could not create socket: {:?}", err);
                return;
            },
            Ok(socket) => socket,
        }
    };

    handle_log_text(
        String::from(LOG_SERVER_NAME),
        LogText {
            timestamp: OffsetDateTime::now_utc().unix_timestamp_nanos(),
            severity: Severity::Status,
            text: String::from("Log server online"),
        },
    );

    loop {
        let result = listen(&socket, 0x00);

        if let Err(error) = result {
            match error {
                NetworkError::ZmqError(error) => handle_log_text(
                    String::from(LOG_SERVER_NAME),
                    LogText {
                        timestamp: OffsetDateTime::now_utc().unix_timestamp_nanos(),
                        severity: Severity::RecoverableError,
                        text: format!("ZeroMQ error: {error}"),
                    },
                ),
                NetworkError::SerialisationError(error) => handle_log_text(
                    String::from(LOG_SERVER_NAME),
                    LogText {
                        timestamp: OffsetDateTime::now_utc().unix_timestamp_nanos(),
                        severity: Severity::RecoverableError,
                        text: format!("Serialisation error: {error}"),
                    },
                ),
            }
        }
    }
}

fn create_listen_socket(context: zmq::Context) -> Result<zmq::Socket, zmq::Error> {
    let socket = context.socket(zmq::SUB)?;
    // subscribe to everything
    socket.set_subscribe(b"")?;
    socket.bind(static_addresses::LOG_SERVER)?;

    Ok(socket)
}

fn listen(socket: &zmq::Socket, receive_flags: i32) -> Result<(), NetworkError> {
    let received_messages = socket
        .recv_multipart(receive_flags)
        .map_err(|err| NetworkError::ZmqError(err))?;

    let _topic = &received_messages[0];
    let encoded = &received_messages[1];

    let message =
        bincode::deserialize(&encoded[..]).map_err(|err| NetworkError::SerialisationError(err))?;

    handle_message(message);

    Ok(())
}

fn handle_message(message: Message) {
    match message {
        Message::Text(sender, text) => handle_log_text(sender, text),
    }
}

fn handle_log_text(sender: Sender, text: LogText) {
    println!(
        "{:>20} - {:<30}: [{:<20?}] {}",
        text.timestamp, sender, text.severity, text.text
    )
}
