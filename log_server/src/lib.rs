pub mod logger;
pub mod logger_mt;

use serde::{Deserialize, Serialize};

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