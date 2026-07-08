use anyhow::{Result, anyhow};

use crate::{
    settings::Settings,
    store::{
        context::StoreCtx,
        ext::StoreExt,
        json::{opts::StoreJsonOpts, state::StoreJsonState},
        server::ServerStore,
    },
};

#[derive(Debug, Clone, Default)]
pub struct JsonStore {}

impl StoreExt for StoreCtx<JsonStore, StoreJsonState, StoreJsonOpts> {
    async fn init(&mut self) -> Result<()> {
        // We have no setup for JSON since we read/write from the file with a new handle each time (no need to manage file handles).
        Ok(())
    }

    async fn settings_fetch(&self) -> Result<Settings> {
        // Fetch current settings from the JSON store.
        let data = self.get_state_data().await?;

        Ok(data.settings.clone())
    }

    async fn settings_save(&mut self, settings: &Settings) -> Result<()> {
        // Update settings in the JSON store.
        {
            let mut state = self.state.write().await;

            state.store.settings = settings.clone();
        }

        // Simply save the entire store.
        self.save_json().await
    }

    async fn srv_fetch_by_id(&self, id: &str) -> Result<Option<ServerStore>> {
        let data = self.get_state_data().await?;

        Ok(data.servers.iter().find(|s| s.id == id).cloned())
    }

    async fn srv_fetch_by_addr(&self, ip: &str, port: u16) -> Result<Option<ServerStore>> {
        let data = self.get_state_data().await?;

        Ok(data
            .servers
            .iter()
            .find(|s| s.ip == ip && s.port == port)
            .cloned())
    }

    async fn srv_fetch_all(&self) -> Result<Vec<ServerStore>> {
        let data = self.get_state_data().await?;

        Ok(data.servers.clone())
    }

    async fn srv_add(&mut self, server: &ServerStore) -> Result<()> {
        // Fetch current settings from the JSON store.
        let data = self.get_state_data().await?;

        // Check if a server with the same ID or IP/port already exists to prevent duplicates.
        if data.servers.iter().any(|s| s.id == server.id) {
            return Err(anyhow!(
                "Server with ID {} or address {}:{} already exists",
                server.id,
                server.ip,
                server.port
            ));
        }

        // Add a server to the JSON store.
        {
            let mut state = self.state.write().await;

            state.store.servers.push(server.clone());
        }

        // Save the entire store after modification.
        self.save_json().await
    }

    async fn srv_update(&mut self, server: &ServerStore) -> Result<()> {
        // Fetch current settings from the JSON store.
        let mut data = self.get_state_data().await?;

        // Update a server in the JSON store.
        if let Some(existing) = data
            .servers
            .iter_mut()
            .find(|s| s.id == server.id || (s.ip == server.ip && s.port == server.port))
        {
            *existing = server.clone();
        } else {
            return Err(anyhow!("Server with ID {} not found for update", server.id));
        }

        // Write state data.
        {
            let mut state = self.state.write().await;

            state.store = data;
        }

        // Save the entire store after modification.
        self.save_json().await
    }

    async fn srv_delete(&mut self, server: &ServerStore) -> Result<()> {
        // Fetch current settings from the JSON store.
        let mut data = self.get_state_data().await?;

        // Delete a server by ID or IP/port from the JSON store.
        if let Some(pos) = data
            .servers
            .iter()
            .position(|s| s.id == server.id || (s.ip == server.ip && s.port == server.port))
        {
            data.servers.remove(pos);
        } else {
            return Err(anyhow!(
                "Server with ID {} not found for deletion",
                server.id
            ));
        }

        // Write state data.
        {
            let mut state = self.state.write().await;

            state.store = data;
        }

        // Save the entire store after modification.
        self.save_json().await
    }
}
