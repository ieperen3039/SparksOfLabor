pub mod log {
    use serde::{Deserialize, Serialize};
    use time::OffsetDateTime;

    pub const VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

    pub const CONNECTION_NAME_LOG_SERVER_SUB: &str = "LogSubscriber";

    pub type Sender = String;

    #[derive(Serialize, Deserialize)]
    pub enum Message {
        Text(Sender, LogText),
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Severity {
        // for least important to most important
        Activity,
        Status,
        EnvironmentIssue,
        RecoverableError,
        FatalError,
    }

    #[derive(Serialize, Deserialize)]
    pub struct LogText {
        pub timestamp: i128,
        pub severity: Severity,
        pub text: String,
    }

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

        pub fn send_status(
            &self,
            text: &str,
        ) {
            self.log(Severity::Status, text)
        }

        pub fn send_debug(
            &self,
            text: &str,
        ) {
            self.log(Severity::Activity, text)
        }

        pub fn log(
            &self,
            severity: Severity,
            text: &str,
        ) {
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

            let send_result =
                bincode::serialize(&log_message).map(|encoded| {
                    let topic_bytes = b"Log";
                    self.socket.send_multipart([topic_bytes.to_vec(), encoded], 0x00)
                });

            match send_result {
                Err(error) => println!(
                    "Could not send log message: serialisation of log message failed : {error}"
                ),
                Ok(Err(error)) => {
                    println!("Could not send log message: sending the log message failed : {error}")
                },
                _ => {},
            }
        }
    }
}
