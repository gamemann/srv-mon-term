pub mod buffer;
pub mod level;

use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use crate::logger::{buffer::LogBufferData, types::level::LogLevel};

pub type LogBuffer = Arc<RwLock<VecDeque<LogBufferData>>>;

pub struct Logger {
    pub is_basic: bool,

    pub levels: Vec<LogLevel>,

    pub buffer: LogBuffer,
    pub max_buffer_size: usize,

    pub path: Option<String>,
}

impl Default for Logger {
    fn default() -> Self {
        Logger {
            is_basic: false,
            levels: vec![
                LogLevel::Info,
                LogLevel::Warn,
                LogLevel::Error,
                LogLevel::Fatal,
            ],
            buffer: Arc::new(RwLock::new(VecDeque::new())),
            max_buffer_size: 10000,
            path: Some("logs/%Y-%m-%d.log".to_string()),
        }
    }
}
