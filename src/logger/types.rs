pub mod buffer;
pub mod level;

use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use crate::{
    logger::{buffer::LogBufferData, level::LogLevel},
    settings::{
        SETTINGS_DEFAULT_LOG_LEVELS, SETTINGS_DEFAULT_LOG_MAX_BUFFER_SIZE,
        SETTINGS_DEFAULT_LOG_PATH,
    },
};

pub type LogBuffer = Arc<RwLock<VecDeque<LogBufferData>>>;

pub struct Logger {
    pub buffer: LogBuffer,

    // These are ONLY used when not logging from the context (i.e., before context is initialized). Once context is initialized, the logger from the context should be used instead.
    pub levels: Vec<LogLevel>,
    pub path: Option<String>,
    pub max_buffer_size: usize,
    pub is_basic: bool,
}

impl Default for Logger {
    fn default() -> Self {
        Logger {
            buffer: Arc::new(RwLock::new(VecDeque::new())),
            path: SETTINGS_DEFAULT_LOG_PATH,
            max_buffer_size: SETTINGS_DEFAULT_LOG_MAX_BUFFER_SIZE,
            is_basic: false,
            levels: SETTINGS_DEFAULT_LOG_LEVELS.to_vec(),
        }
    }
}
