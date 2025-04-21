use crate::logger::Logger;
use crate::Severity;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct LoggerMt {
    inner: Arc<Mutex<Logger>>,
}

impl LoggerMt {
    pub fn new(
        this_name: &str,
        context: zmq::Context,
        endpoint: String,
    ) -> Result<LoggerMt, zmq::Error> {
        let logger = Logger::new(this_name, context, endpoint)?;
        Ok(LoggerMt {
            inner: Arc::new(Mutex::new(logger)),
        })
    }

    pub fn send_status(&self, text: &str) {
        self.log(Severity::Status, text)
    }

    pub fn send_debug(&self, text: &str) {
        self.log(Severity::Activity, text)
    }

    pub fn log(&self, severity: Severity, text: &str) {
        let logger = self.inner.lock().unwrap();
        logger.log(severity, text);
    }
}
