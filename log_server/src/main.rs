use sol_address_server::static_addresses;
use sol_log_server::log::*;
use sol_network_lib::network::{self, NetworkError};
use time::OffsetDateTime;

const LOG_SERVER_NAME: &str = "Log server";

fn main() {
    let context = zmq::Context::new();

    let socket = context.socket(zmq::SUB).expect("Could not create socket");
    socket
        .bind(static_addresses::LOG_SERVER)
        .expect("Could not bind socket");

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

fn listen(
    socket: &zmq::Socket,
    receive_flags: i32,
) -> Result<(), NetworkError> {
    let message = network::await_receive(socket, receive_flags)?;
    println!("Received something!");
    handle_message(message);

    Ok(())
}

fn handle_message(message: Message) {
    match message {
        Message::Text(sender, text) => handle_log_text(sender, text),
    }
}

fn handle_log_text(
    sender: Sender,
    text: LogText,
) {
    println!(
        "{} - {:<30}: [{:<20?}] {}",
        sender, text.timestamp, text.severity, text.text
    )
}
