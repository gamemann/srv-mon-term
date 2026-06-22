use std::{collections::VecDeque, sync::Arc};

use tokio::sync::RwLock;

use crate::logger::types::{Logger, level::LogLevel};

impl Logger {
    pub fn new(levels: Vec<LogLevel>, path: Option<String>, is_basic: bool) -> Self {
        Logger {
            is_basic,
            levels,
            buffer: Arc::new(RwLock::new(VecDeque::new())),
            max_buffer_size: Default::default(),
            path,
        }
    }
}
