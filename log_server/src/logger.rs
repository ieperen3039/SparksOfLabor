use crate::{LogText, Message, Sender, Severity};
use time::OffsetDateTime;

pub struct Logger {
    owner_name: Sender,
    socket: zmq::Socket,
}

impl Logger {
    pub fn new(
        this_name: &str,
        context: zmq::Context,
        endpoint: String,
    ) -> Result<Logger, zmq::Error> {
        let socket = context.socket(zmq::PUB)?;
        socket.connect(&endpoint)?;

        Ok(Logger {
            owner_name: String::from(this_name),
            socket,
        })
    }

    pub fn send_status(&self, text: &str) {
        self.log(Severity::Status, text)
    }

    pub fn send_debug(&self, text: &str) {
        self.log(Severity::Activity, text)
    }

    pub fn log(&self, severity: Severity, text: &str) {
        let timestamp = OffsetDateTime::now_utc().unix_timestamp_nanos();

        println!("{:<20?}: {}", severity, text);

        let log_message = Message::Text(
            self.owner_name.clone(),
            LogText {
                timestamp,
                severity,
                text: String::from(text),
            },
        );

        let serialized = bincode::serialize(&log_message);

        match serialized {
            Err(error) => {
                println!(
                    "Could not send log message: serialisation of log message failed : {error}"
                );
                return;
            }
            Ok(encoded) => {
                let topic_bytes = b"Log";
                let send_result = self
                    .socket
                    .send_multipart([topic_bytes.to_vec(), encoded], 0x00);

                if let Err(error) = send_result {
                    println!("Could not send log message: sending the log message failed : {error}")
                }
            }
        }
    }
}