pub mod store;

use std::path::PathBuf;

use anyhow::{Result, anyhow, bail};
use tokio::fs;

use crate::store::types::json::store::JsonStoreFormat;

#[derive(Debug, Clone, Default)]
pub struct JsonStore {
    pub store_path: String,
    pub store_fmt: JsonStoreFormat,

    pub rw_lock: bool,
}

impl JsonStore {
    pub fn new(store_path: String) -> Self {
        let mut path = PathBuf::from(&store_path);

        path.add_extension("json");

        Self {
            store_path: path.to_string_lossy().to_string(),
            store_fmt: JsonStoreFormat::default(),
            rw_lock: false,
        }
    }

    pub async fn get_json_str(&self) -> Result<String> {
        let path = self.get_path().await?;

        if path.exists() {
            match fs::read_to_string(path).await {
                Ok(content) => return Ok(content),
                Err(e) => bail!("Failed to read JSON store: {}", e),
            }
        }

        // Create file if it doesn't exist
        fs::write(path, "{}").await?;

        Ok("{}".to_string())
    }

    pub async fn get_json(&mut self) -> Result<()> {
        let json_str = self.get_json_str().await?;

        let store_format: JsonStoreFormat = serde_json::from_str(&json_str)
            .map_err(|e| anyhow!("Failed to parse JSON store: {}", e))?;

        self.store_fmt = store_format;

        Ok(())
    }

    pub async fn save_json_str(&self, json: &str) -> Result<()> {
        let path = self.get_path().await?;

        match fs::write(path, json).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // If the file doesn't exist, create it and write the JSON contents.
                /*
                if e.kind() == std::io::ErrorKind::NotFound {
                    fs::write(path, json).await.map_err(|e| {
                        anyhow!("Failed to create JSON file after not found: {}", e)
                    })?;

                    return Ok(());
                }
                */

                Err(anyhow!(
                    "Failed to write JSON store to existing file: {}",
                    e
                ))
            }
        }
    }

    pub async fn save_json(&self) -> Result<()> {
        // Convert store format to JSON string and save to the file.
        let json_str = serde_json::to_string_pretty(&self.store_fmt)
            .map_err(|e| anyhow!("Failed to serialize JSON store: {}", e))?;

        self.save_json_str(&json_str).await
    }

    async fn get_path(&self) -> Result<PathBuf> {
        let path_full = format!("{}.json", self.store_path);

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
