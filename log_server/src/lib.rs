use serde::{Serialize, Deserialize};

pub const VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

pub const CONNECTION_NAME_LOG_SERVER_SUB : &str = "LogSubscriber";

#[derive(Serialize, Deserialize)]
pub enum LoggingMessage {
    Text(LogText)
}

#[derive(Serialize, Deserialize)]
pub enum LogType {
    // for least important to most important
    Activity,
    Status,
    EnvironmentIssue,
    RecoverableError,
    FatalError,
}

#[derive(Serialize, Deserialize)]
pub struct LogText {
    pub timestamp : time::OffsetDateTime,
    pub level : LogType,
    pub text : String,
}