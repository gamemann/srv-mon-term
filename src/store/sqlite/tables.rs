use anyhow::{Result, anyhow};

use crate::store::types::sqlite::SqliteStore;

pub const TABLE_SETTINGS: &str = "CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
)";

pub const TABLE_SERVER: &str = "CREATE TABLE IF NOT EXISTS servers (
    id TEXT PRIMARY KEY,
    name TEXT,
    ip TEXT NOT NULL,
    port INTEGER NOT NULL,
    port_query INTEGER,
    query_interval INTEGER DEFAULT 1000,
    query_timeout INTEGER DEFAULT 5000,
    query_type INTEGER DEFAULT 0,
    latency_interval INTEGER,
    latency_timeout INTEGER,
    latency_type INTEGER DEFAULT 0,
    latency_history_size INTEGER DEFAULT 100,

    UNIQUE(ip, port)
)";

impl SqliteStore {
    pub async fn create_tables(&self) -> Result<()> {
        if let Some(conn) = &self.conn {
            conn.execute(TABLE_SETTINGS, [])
                .map_err(|e| anyhow!("Failed to create settings table: {}", e))?;
            conn.execute(TABLE_SERVER, [])
                .map_err(|e| anyhow!("Failed to create servers table: {}", e))?;
        }
        Ok(())
    }
}
