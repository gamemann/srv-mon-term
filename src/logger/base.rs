use std::{
    collections::VecDeque,
    fs,
    path::Path,
    sync::{Arc, RwLock},
};

use crate::logger::types::{Logger, level::LogLevel};

impl Logger {
    pub async fn new(levels: Vec<LogLevel>, path: Option<String>, is_basic: bool) -> Self {
        // If we have a path, try to create the directory if it doesn't exist
        if let Some(ref path_fmt) = path {
            if let Some(parent) = Path::new(path_fmt).parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Failed to create log file parent directories: {}", e);
                }
            }
        }

        Logger {
            is_basic,
            levels,
            buffer: Arc::new(RwLock::new(VecDeque::new())),
            max_buffer_size: 100,
            path,
        }
    }
}
