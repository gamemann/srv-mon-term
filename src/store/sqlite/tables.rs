use anyhow::{Result, anyhow};

use crate::store::{
    StoreCtx,
    sqlite::{base::SqliteStore, opts::StoreSqliteOpts, state::StoreSqliteState},
};

pub const TABLE_SETTINGS: &str = "CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
)";

pub const TABLE_SERVER: &str = "CREATE TABLE IF NOT EXISTS servers (
    id TEXT PRIMARY KEY,
    ip TEXT NOT NULL,
    port INTEGER NOT NULL,
    display_name TEXT,
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

impl StoreCtx<SqliteStore, StoreSqliteState, StoreSqliteOpts> {
    pub async fn create_tables(&self) -> Result<()> {
        let conn = self.get_conn_lock().await?;

        let conn = conn
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on SQLite connection: {}", e))?;

        conn.execute(TABLE_SETTINGS, [])
            .map_err(|e| anyhow!("Failed to create settings table: {}", e))?;
        conn.execute(TABLE_SERVER, [])
            .map_err(|e| anyhow!("Failed to create servers table: {}", e))?;

        Ok(())
    }
}
