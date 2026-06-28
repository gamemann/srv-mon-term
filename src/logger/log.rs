use std::path::PathBuf;
use tokio::fs;

use anyhow::{Result, anyhow};

use crate::logger::{
    buffer::LogBufferData,
    types::{Logger, level::LogLevel},
};

use tokio::io::AsyncWriteExt;

impl Logger {
    pub async fn log_msg(&mut self, level: LogLevel, msg: String) -> Result<()> {
        if !self.levels.contains(&level) {
            return Ok(());
        }

        let msg_base = format!("[{}] {}", level.to_string().to_uppercase(), msg);

        // Print the base message to the terminal if basic mode is enabled. Otherwise, push to buffer to display in the TUI.
        if self.is_basic {
            println!("{}", msg_base.clone());
        } else {
            // Acquire write lock from buffer and push log message.
            let mut buffer = self.buffer.write().await;

            // Check if the buffer is full before pushing a new message. If it is, pop the oldest message to make room for the new one.
            if buffer.len() >= self.max_buffer_size {
                buffer.pop_front();
            }

            // Push the new log message to the buffer.
            let buff_data = LogBufferData::new(level.clone(), msg);

            buffer.push_back(buff_data);
        }

        if let Some(path_fmt) = &self.path {
            // First let's format the path with the current date and time if any.
            let now = chrono::Local::now();

            let path = now.format(path_fmt).to_string();
            let path = PathBuf::from(path);

            // Open the file in append mode, creating it if it doesn't exist.
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await
                .map_err(|e| anyhow!("Failed to open or create log file: {}", e))?;

            // Format the log message with a timestamp.
            let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

            let log_msg = format!("[{}] {}\n", timestamp, msg_base.clone());

            // Attempt to write the log message to the file.
            file.write_all(log_msg.as_bytes())
                .await
                .map_err(|e| anyhow!("Failed to write to log file: {}", e))?;
        }

        Ok(())
    }
}
