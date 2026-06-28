use chrono::{DateTime, Local};

use crate::logger::level::LogLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogBufferData {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Local>,
}

impl LogBufferData {
    pub fn new(level: LogLevel, message: String) -> Self {
        LogBufferData {
            level,
            message,
            timestamp: Local::now(),
        }
    }
}
