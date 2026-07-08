use std::path::PathBuf;

use anyhow::{Result, anyhow, bail};
use tokio::fs;

use crate::store::{
    Store, StoreCtx,
    json::{base::JsonStore, data::JsonStateData, opts::StoreJsonOpts, state::StoreJsonState},
};

impl StoreCtx<JsonStore, StoreJsonState, StoreJsonOpts> {
    pub async fn get_store_path(&self) -> String {
        let path_fmt = Store::fmt_store_path(self.opts.path.clone());

        let mut path = PathBuf::from(&path_fmt);

        path.add_extension("json");

        path.to_string_lossy().to_string()
    }

    pub async fn get_json_str(&self) -> Result<String> {
        let path = self.get_store_path().await;

        if PathBuf::from(&path).exists() {
            match fs::read_to_string(&path).await {
                Ok(content) => return Ok(content),
                Err(e) => bail!("Failed to read JSON store: {}", e),
            }
        }

        // Create file if it doesn't exist
        fs::write(&path, "{}").await?;

        Ok("{}".to_string())
    }

    pub async fn get_state_data(&self) -> Result<JsonStateData> {
        let json_str = self.get_json_str().await?;

        serde_json::from_str(&json_str).map_err(|e| anyhow!("Failed to parse JSON store: {}", e))
    }

    pub async fn save_json_str(&self, json: &str) -> Result<()> {
        let path = self.get_store_path().await;

        match fs::write(&path, json).await {
            Ok(_) => Ok(()),
            Err(e) => bail!("Failed to write JSON store to existing file: {}", e),
        }
    }

    pub async fn save_json(&self) -> Result<()> {
        // Convert store format to JSON string and save to the file.
        let json_str = {
            let state = self.state.read().await;

            serde_json::to_string_pretty(&state.store)
                .map_err(|e| anyhow!("Failed to serialize JSON store: {}", e))?
        };

        self.save_json_str(&json_str).await
    }

    async fn get_path(&self) -> Result<PathBuf> {
        let path_full = format!("{}.json", self.opts.path);

        let path = PathBuf::from(path_full);

        // Create parent directories if they don't exist.
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                anyhow!("Failed to create parent directories for JSON store: {}", e)
            })?;
        }

        Ok(path)
    }
}
