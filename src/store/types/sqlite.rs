use std::path::PathBuf;

use anyhow::{Result, anyhow};
use rusqlite::Connection;

#[derive(Debug, Default)]

pub struct SqliteStore {
    pub store_path: String,
    pub conn: Option<Connection>,
}

impl SqliteStore {
    pub fn new(store_path: String) -> Self {
        let mut path = PathBuf::from(&store_path);

        path.add_extension("db");

        Self {
            store_path: path.to_string_lossy().to_string(),
            conn: None,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        let path = PathBuf::from(&self.store_path);

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    anyhow!(
                        "Failed to create parent directories for SQLite store: {}",
                        e
                    )
                })?;
            }
        }

        let conn = Connection::open(&self.store_path)
            .map_err(|e| anyhow!("Failed to connect to SQLite database: {}", e))?;

        self.conn = Some(conn);

        Ok(())
    }
}
