use std::{
    collections::VecDeque,
    fs,
    path::Path,
    sync::{Arc, RwLock},
};

use crate::{
    logger::{level::LogLevel, types::Logger},
    settings::SETTINGS_DEFAULT_LOG_LEVELS,
};

impl Logger {
    pub async fn new(
        path: Option<String>,
        max_buffer_size: usize,
        is_basic: bool,
        levels: Option<Vec<LogLevel>>,
    ) -> Self {
        // If we have a path, try to create the directory if it doesn't exist
        if let Some(ref path_fmt) = path {
            if let Some(parent) = Path::new(path_fmt).parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Failed to create log file parent directories: {}", e);
                }
            }
        }

        Logger {
            buffer: Arc::new(RwLock::new(VecDeque::new())),
            path,
            max_buffer_size,
            is_basic,
            levels: levels.unwrap_or(SETTINGS_DEFAULT_LOG_LEVELS.to_vec()),
        }
    }

    pub fn set_path(&mut self, path: Option<String>) {
        self.path = path;
    }

    pub fn set_levels(&mut self, levels: Vec<LogLevel>) {
        self.levels = levels;
    }

    pub fn set_max_buffer_size(&mut self, max_buffer_size: usize) {
        self.max_buffer_size = max_buffer_size;
    }

    pub fn set_is_basic(&mut self, is_basic: bool) {
        self.is_basic = is_basic;
    }
}
