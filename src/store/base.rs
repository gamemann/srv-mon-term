use std::{env::home_dir, sync::Arc};

use anyhow::{Result, anyhow};

use crate::{
    context::Context,
    store::{
        context::StoreCtx,
        types::{Store, json::JsonStore, sqlite::SqliteStore},
    },
};

impl Store {
    fn fmt_store_path(store_path: String) -> String {
        // Retrieve the home directory if possible.
        let home_dir = home_dir();

        let home_path = if let Some(dir) = home_dir {
            dir.to_str().map(|s| s.to_string())
        } else {
            None
        };

        let mut store_path = store_path;

        // If we have a home directory, replace "~/" with the home directory path.
        if let Some(home) = home_path {
            if store_path.starts_with("~/") {
                store_path = store_path.replacen("~/", &format!("{home}/"), 1);
            }
        }

        store_path
    }

    pub fn new(store_type: &str, store_path: String) -> Result<Self> {
        let store_path = Self::fmt_store_path(store_path);

        match store_type {
            "json" => Ok(Self::new_json(store_path)),
            "sqlite" => Ok(Self::new_sqlite(store_path)),
            _ => Err(anyhow!("Unsupported store type: {}", store_type)),
        }
    }

    pub fn get_store_name(&self) -> &str {
        match self {
            Store::Json(_) => "json",
            Store::Sqlite(_) => "sqlite",
        }
    }

    pub fn set_context(&mut self, ctx: Context) {
        match self {
            Store::Json(store_ctx) => store_ctx.ctx = Some(Arc::downgrade(&ctx)),
            Store::Sqlite(store_ctx) => store_ctx.ctx = Some(Arc::downgrade(&ctx)),
        }
    }

    fn new_json(store_path: String) -> Self {
        Store::Json(StoreCtx::new(JsonStore::new(store_path)))
    }

    fn new_sqlite(store_path: String) -> Self {
        Store::Sqlite(StoreCtx::new(SqliteStore::new(store_path)))
    }
}
