use std::collections::HashMap;

use anyhow::{Result, anyhow, bail};
use rusqlite::named_params;

use crate::{
    settings::Settings,
    store::{
        context::StoreCtx,
        ext::StoreExt,
        server::ServerStore,
        sqlite::queries::{
            SQL_SETTINGS_FETCH, SQL_SETTINGS_SAVE, SQL_SRV_DELETE, SQL_SRV_FETCH_ALL,
            SQL_SRV_FETCH_BY_ADDR, SQL_SRV_FETCH_BY_ID, SQL_SRV_INSERT, SQL_SRV_UPDATE,
        },
        types::sqlite::SqliteStore,
    },
};

impl StoreExt for StoreCtx<SqliteStore> {
    async fn init(&mut self) -> Result<()> {
        let mut store = self.store.lock().await;

        // Attempt to connect to the SQLite database. If the file doesn't exist, it will be created.
        store.connect().await?;

        // Create tables if they don't exist.
        store.create_tables().await?;

        Ok(())
    }

    async fn settings_fetch(&mut self) -> Result<Settings> {
        let store = self.store.lock().await;

        if let Some(conn) = &store.conn {
            let mut stmt = conn
                .prepare(SQL_SETTINGS_FETCH)
                .map_err(|e| anyhow!("Failed to prepare statement: {}", e))?;

            // Execute query and fetch results.
            let mut rows = stmt
                .query([])
                .map_err(|e| anyhow!("Failed to execute query: {}", e))?;

            let mut settings_map = HashMap::new();

            while let Some(row) = rows
                .next()
                .map_err(|e| anyhow!("Failed to fetch row: {}", e))?
            {
                let key: String = row
                    .get(0)
                    .map_err(|e| anyhow!("Failed to get key from row: {}", e))?;

                let value: String = row
                    .get(1)
                    .map_err(|e| anyhow!("Failed to get value from row: {}", e))?;

                settings_map.insert(key, value);
            }

            // Convert settings from the map.
            Settings::from_map(&settings_map)
                .map_err(|e| anyhow!("Failed to convert settings from map: {}", e))
        } else {
            Err(anyhow!("No database connection available"))
        }
    }

    async fn settings_save(&mut self, settings: &Settings) -> Result<()> {
        // Convert settings to a map.
        let settings_map = settings.to_map();

        let store = self.store.lock().await;

        if let Some(conn) = &store.conn {
            let mut stmt = conn
                .prepare(SQL_SETTINGS_SAVE)
                .map_err(|e| anyhow!("Failed to prepare statement: {}", e))?;

            // Insert or update each setting in the database.
            for (key, value) in settings_map {
                stmt.execute([key, value])
                    .map_err(|e| anyhow!("Failed to execute statement: {}", e))?;
            }

            Ok(())
        } else {
            Err(anyhow!("No database connection available"))
        }
    }

    async fn srv_fetch_by_id(&mut self, id: &str) -> Result<Option<ServerStore>> {
        let store = self.store.lock().await;

        if let Some(conn) = &store.conn {
            let mut stmt = conn
                .prepare(SQL_SRV_FETCH_BY_ID)
                .map_err(|e| anyhow!("Failed to prepare statement: {}", e))?;

            let mut rows = stmt
                .query([id])
                .map_err(|e| anyhow!("Failed to execute query: {}", e))?;

            if let Some(row) = rows
                .next()
                .map_err(|e| anyhow!("Failed to fetch row: {}", e))?
            {
                // Convert the row into a Server struct.
                let server = ServerStore {
                    id: id.to_string(),
                    ip: row
                        .get(1)
                        .map_err(|e| anyhow!("Failed to get ip from row: {}", e))?,
                    port: row
                        .get(2)
                        .map_err(|e| anyhow!("Failed to get port from row: {}", e))?,
                    display_name: row
                        .get(3)
                        .map_err(|e| anyhow!("Failed to get display_name from row: {}", e))?,
                    port_query: row
                        .get(4)
                        .map_err(|e| anyhow!("Failed to get port_query from row: {}", e))?,
                    query_interval: row
                        .get::<_, i64>(5)
                        .map_err(|e| anyhow!("Failed to get query_interval from row: {}", e))?
                        as u64,
                    query_timeout: row
                        .get::<_, i64>(6)
                        .map_err(|e| anyhow!("Failed to get query_timeout from row: {}", e))?
                        as u64,
                    query_type: row
                        .get::<_, i32>(7)
                        .map_err(|e| anyhow!("Failed to get query_type from row: {}", e))?
                        .try_into()
                        .map_err(|e| anyhow!("Failed to convert query_type from row: {}", e))?,
                    latency_interval: row
                        .get::<_, Option<i64>>(8)
                        .map_err(|e| anyhow!("Failed to get latency_interval from row: {}", e))?
                        .map(|v| v as u64),
                    latency_timeout: row
                        .get::<_, Option<i64>>(9)
                        .map_err(|e| anyhow!("Failed to get latency_timeout from row: {}", e))?
                        .map(|v| v as u64),
                    latency_type: row
                        .get::<_, i32>(10)
                        .map_err(|e| anyhow!("Failed to get latency_type from row: {}", e))?
                        .try_into()
                        .map_err(|e| anyhow!("Failed to convert latency_type from row: {}", e))?,
                    latency_history_size: row.get::<_, u32>(11).map_err(|e| {
                        anyhow!("Failed to get latency_history_size from row: {}", e)
                    })? as usize,

                    ..Default::default()
                };

                Ok(Some(server))
            } else {
                Ok(None)
            }
        } else {
            bail!("No database connection available")
        }
    }

    async fn srv_fetch_by_addr(&mut self, ip: &str, port: u16) -> Result<Option<ServerStore>> {
        let store = self.store.lock().await;

        if let Some(conn) = &store.conn {
            let mut stmt = conn
                .prepare(SQL_SRV_FETCH_BY_ADDR)
                .map_err(|e| anyhow!("Failed to prepare statement: {}", e))?;

            let mut rows = stmt
                .query([ip, &port.to_string()])
                .map_err(|e| anyhow!("Failed to execute query: {}", e))?;

            if let Some(row) = rows
                .next()
                .map_err(|e| anyhow!("Failed to fetch row: {}", e))?
            {
                // Convert the row into a Server struct.
                let server = ServerStore {
                    id: row
                        .get(0)
                        .map_err(|e| anyhow!("Failed to get id from row: {}", e))?,
                    ip: ip.to_string(),
                    port,
                    display_name: row
                        .get(1)
                        .map_err(|e| anyhow!("Failed to get display_name from row: {}", e))?,
                    port_query: row
                        .get(2)
                        .map_err(|e| anyhow!("Failed to get port_query from row: {}", e))?,
                    query_interval: row
                        .get::<_, i64>(3)
                        .map_err(|e| anyhow!("Failed to get query_interval from row: {}", e))?
                        as u64,
                    query_timeout: row
                        .get::<_, i64>(4)
                        .map_err(|e| anyhow!("Failed to get query_timeout from row: {}", e))?
                        as u64,
                    query_type: row
                        .get::<_, i32>(5)
                        .map_err(|e| anyhow!("Failed to get query_type from row: {}", e))?
                        .try_into()
                        .map_err(|e| anyhow!("Failed to convert query_type from row: {}", e))?,
                    latency_interval: row
                        .get::<_, Option<i64>>(6)
                        .map_err(|e| anyhow!("Failed to get latency_interval from row: {}", e))?
                        .map(|v| v as u64),
                    latency_timeout: row
                        .get::<_, Option<i64>>(7)
                        .map_err(|e| anyhow!("Failed to get latency_timeout from row: {}", e))?
                        .map(|v| v as u64),
                    latency_type: row
                        .get::<_, i32>(8)
                        .map_err(|e| anyhow!("Failed to get latency_type from row: {}", e))?
                        .try_into()
                        .map_err(|e| anyhow!("Failed to convert latency_type from row: {}", e))?,
                    latency_history_size: row.get::<_, u32>(9).map_err(|e| {
                        anyhow!("Failed to get latency_history_size from row: {}", e)
                    })? as usize,

                    ..Default::default()
                };

                Ok(Some(server))
            } else {
                Ok(None)
            }
        } else {
            bail!("No database connection available")
        }
    }

    async fn srv_fetch_all(&mut self) -> Result<Vec<ServerStore>> {
        let store = self.store.lock().await;

        if let Some(conn) = &store.conn {
            let mut stmt = conn
                .prepare(SQL_SRV_FETCH_ALL)
                .map_err(|e| anyhow!("Failed to prepare statement: {}", e))?;

            let mut rows = stmt
                .query([])
                .map_err(|e| anyhow!("Failed to execute query: {}", e))?;

            let mut servers = Vec::new();

            while let Some(row) = rows
                .next()
                .map_err(|e| anyhow!("Failed to fetch row: {}", e))?
            {
                // Convert the row into a ServerStore struct.
                let server = ServerStore {
                    id: row
                        .get(0)
                        .map_err(|e| anyhow!("Failed to get id from row: {}", e))?,
                    ip: row
                        .get(1)
                        .map_err(|e| anyhow!("Failed to get ip from row: {}", e))?,
                    port: row
                        .get(2)
                        .map_err(|e| anyhow!("Failed to get port from row: {}", e))?,
                    display_name: row
                        .get(3)
                        .map_err(|e| anyhow!("Failed to get display_name from row: {}", e))?,
                    port_query: row
                        .get(4)
                        .map_err(|e| anyhow!("Failed to get port_query from row: {}", e))?,
                    query_interval: row
                        .get::<_, i64>(5)
                        .map_err(|e| anyhow!("Failed to get query_interval from row: {}", e))?
                        as u64,
                    query_timeout: row
                        .get::<_, i64>(6)
                        .map_err(|e| anyhow!("Failed to get query_timeout from row: {}", e))?
                        as u64,
                    query_type: row
                        .get::<_, i32>(7)
                        .map_err(|e| anyhow!("Failed to get query_type from row: {}", e))?
                        .try_into()
                        .map_err(|e| anyhow!("Failed to convert query_type from row: {}", e))?,
                    latency_interval: row
                        .get::<_, Option<i64>>(8)
                        .map_err(|e| anyhow!("Failed to get latency_interval from row: {}", e))?
                        .map(|v| v as u64),
                    latency_timeout: row
                        .get::<_, Option<i64>>(9)
                        .map_err(|e| anyhow!("Failed to get latency_timeout from row: {}", e))?
                        .map(|v| v as u64),
                    latency_type: row
                        .get::<_, i32>(10)
                        .map_err(|e| anyhow!("Failed to get latency_type from row: {}", e))?
                        .try_into()
                        .map_err(|e| anyhow!("Failed to convert latency_type from row: {}", e))?,
                    latency_history_size: row.get::<_, u32>(11).map_err(|e| {
                        anyhow!("Failed to get latency_history_size from row: {}", e)
                    })? as usize,

                    ..Default::default()
                };

                servers.push(server);
            }

            Ok(servers)
        } else {
            bail!("No database connection available")
        }
    }

    async fn srv_add(&mut self, server: &ServerStore) -> Result<()> {
        let store = self.store.lock().await;

        // If the ID is empty, return an error. The ID is required for inserting a server into the database.
        if server.id.is_empty() {
            bail!("Server ID is empty. Cannot add server to the database.");
        }

        // Add a server to the SQLite store.
        if let Some(conn) = &store.conn {
            conn.execute(
                SQL_SRV_INSERT,
                named_params![
                    ":id": server.id,
                    ":ip": server.ip,
                    ":port": server.port,
                    ":display_name": server.display_name,
                    ":port_query": server.port_query,
                    ":query_interval": server.query_interval as i64,
                    ":query_timeout": server.query_timeout as i64,
                    ":query_type": server.query_type.clone() as i32,
                    ":latency_interval": server.latency_interval.map(|v| v as i64),
                    ":latency_timeout": server.latency_timeout.map(|v| v as i64),
                    ":latency_type": server.latency_type.clone() as i32,
                    ":latency_history_size": server.latency_history_size as i64,
                ],
            )
            .map_err(|e| anyhow!("Failed to execute statement: {}", e))?;

            Ok(())
        } else {
            bail!("No database connection available")
        }
    }

    async fn srv_update(&mut self, server: &ServerStore) -> Result<()> {
        // Update a server in the SQLite store.
        let store = self.store.lock().await;

        if let Some(conn) = &store.conn {
            conn.execute(
                SQL_SRV_UPDATE,
                named_params![
                    ":ip": server.ip,
                    ":port": server.port,
                    ":display_name": server.display_name,
                    ":port_query": server.port_query,
                    ":query_interval": server.query_interval as i64,
                    ":query_timeout": server.query_timeout as i64,
                    ":query_type": server.query_type.clone() as i32,
                    ":latency_interval": server.latency_interval.map(|v| v as i64),
                    ":latency_timeout": server.latency_timeout.map(|v| v as i64),
                    ":latency_type": server.latency_type.clone() as i32,
                    ":latency_history_size": server.latency_history_size as i64,
                    ":id": server.id,
                ],
            )
            .map_err(|e| anyhow!("Failed to execute statement: {}", e))?;

            Ok(())
        } else {
            bail!("No database connection available")
        }
    }

    async fn srv_delete(&mut self, server: &ServerStore) -> Result<()> {
        let store = self.store.lock().await;

        if let Some(conn) = &store.conn {
            conn.execute(
                SQL_SRV_DELETE,
                named_params![
                    ":id": server.id,
                    ":ip": server.ip,
                    ":port": server.port,
                ],
            )
            .map_err(|e| anyhow!("Failed to execute statement: {}", e))?;

            Ok(())
        } else {
            bail!("No database connection available")
        }
    }
}
