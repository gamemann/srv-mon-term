use anyhow::{Result, anyhow};

use crate::{
    server::types::Server,
    settings::Settings,
    store::{context::StoreCtx, ext::StoreExt, types::json::JsonStore},
};

impl StoreExt for StoreCtx<JsonStore> {
    async fn init(&mut self) -> Result<()> {
        // We have no setup for JSON since we read/write from the file with a new handle each time (no need to manage file handles).
        Ok(())
    }

    async fn settings_fetch(&mut self) -> Result<Settings> {
        let mut store = self.store.lock().await;

        // Fetch current settings from the JSON store.
        store.get_json().await?;

        Ok(store.store_fmt.settings.clone())
    }

    async fn settings_save(&mut self, settings: &Settings) -> Result<()> {
        let mut store = self.store.lock().await;

        // Update settings in the JSON store.
        store.store_fmt.settings = settings.clone();

        // Simply save the entire store.
        store.save_json().await
    }

    async fn srv_fetch_by_id(&mut self, id: &str) -> Result<Option<Server>> {
        // Do a quick read of the JSON store to ensure we have the latest data.
        let mut store = self.store.lock().await;
        store.get_json().await?;

        store
            .store_fmt
            .servers
            .iter()
            .find(|s| s.id == Some(id.to_string()))
            .cloned()
            .map_or_else(|| Ok(None), |server| Ok(Some(server)))
    }

    async fn srv_fetch_by_addr(&mut self, ip: &str, port: u16) -> Result<Option<Server>> {
        // Do a quick read of the JSON store to ensure we have the latest data.
        let mut store = self.store.lock().await;
        store.get_json().await?;

        store
            .store_fmt
            .servers
            .iter()
            .find(|s| s.ip == ip && s.port == port)
            .cloned()
            .map_or_else(|| Ok(None), |server| Ok(Some(server)))
    }

    async fn srv_fetch_all(&mut self) -> Result<Vec<Server>> {
        // Do a quick read of the JSON store to ensure we have the latest data.
        let mut store = self.store.lock().await;

        store.get_json().await?;

        Ok(store.store_fmt.servers.clone())
    }

    async fn srv_add(&mut self, server: &Server) -> Result<()> {
        // Fetch current settings from the JSON store.
        let mut store = self.store.lock().await;

        store.get_json().await?;

        // Check if a server with the same ID or IP/port already exists to prevent duplicates.
        if store.store_fmt.servers.iter().any(|s| {
            (s.id.is_some() && s.id == server.id) || (s.ip == server.ip && s.port == server.port)
        }) {
            return Err(anyhow!(
                "Server with ID {} or address {}:{} already exists",
                server.id.as_deref().unwrap_or("unknown"),
                server.ip,
                server.port
            ));
        }

        // Add a server to the JSON store.
        store.store_fmt.servers.push(server.clone());

        // Save the entire store after modification.
        store.save_json().await
    }

    async fn srv_update(&mut self, server: &Server) -> Result<()> {
        // Fetch current settings from the JSON store.
        let mut store = self.store.lock().await;

        store.get_json().await?;

        // Update a server in the JSON store.
        if let Some(existing) = store.store_fmt.servers.iter_mut().find(|s| {
            (s.id.is_some() && s.id == server.id) || (s.ip == server.ip && s.port == server.port)
        }) {
            *existing = server.clone();
        } else {
            return Err(anyhow!(
                "Server with ID {} not found for update",
                server.id.as_deref().unwrap_or("unknown")
            ));
        }

        // Save the entire store after modification.
        store.save_json().await
    }

    async fn srv_delete(&mut self, server: &Server) -> Result<()> {
        // Fetch current settings from the JSON store.
        let mut store = self.store.lock().await;

        store.get_json().await?;

        // Delete a server by ID or IP/port from the JSON store.
        if let Some(pos) = store.store_fmt.servers.iter().position(|s| {
            (s.id.is_some() && s.id == server.id) || (s.ip == server.ip && s.port == server.port)
        }) {
            store.store_fmt.servers.remove(pos);
        } else {
            return Err(anyhow!(
                "Server with ID {} not found for deletion",
                server.id.as_deref().unwrap_or("unknown")
            ));
        }

        // Save the entire store after modification.
        store.save_json().await
    }
}
