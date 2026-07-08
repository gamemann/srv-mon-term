use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{Result, anyhow};
use rusqlite::Connection;

use crate::store::{
    Store, StoreCtx,
    sqlite::{base::SqliteStore, opts::StoreSqliteOpts, state::StoreSqliteState},
};

impl StoreCtx<SqliteStore, StoreSqliteState, StoreSqliteOpts> {
    pub async fn get_store_path(&self) -> String {
        let path_fmt = Store::fmt_store_path(self.opts.path.clone());

        let mut path = PathBuf::from(&path_fmt);

        path.add_extension("db");

        path.to_string_lossy().to_string()
    }

    pub async fn get_conn_lock(&self) -> Result<Arc<Mutex<Connection>>> {
        let state = self.state.read().await;

        state
            .conn
            .clone()
            .ok_or_else(|| anyhow!("SQLite connection is not established"))
    }

    pub async fn connect(&mut self) -> Result<()> {
        let path = PathBuf::from(&self.get_store_path().await);

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

        let conn = Connection::open(&path)
            .map_err(|e| anyhow!("Failed to connect to SQLite database: {}", e))?;

        // Write connection handle to state.
        {
            let mut state = self.state.write().await;

            state.conn = Some(Arc::new(Mutex::new(conn)));
        }

        Ok(())
    }
}
