use anyhow::Result;

use crate::{
    settings::Settings,
    store::{server::ServerStore, types::Store},
};

#[allow(async_fn_in_trait)]
pub trait StoreExt {
    async fn init(&mut self) -> Result<()>;

    async fn settings_fetch(&self) -> Result<Settings>;
    async fn settings_save(&mut self, settings: &Settings) -> Result<()>;

    async fn srv_fetch_by_id(&self, id: &str) -> Result<Option<ServerStore>>;
    async fn srv_fetch_by_addr(&self, ip: &str, port: u16) -> Result<Option<ServerStore>>;
    async fn srv_fetch_all(&self) -> Result<Vec<ServerStore>>;

    async fn srv_add(&mut self, server: &ServerStore) -> Result<()>;
    async fn srv_update(&mut self, server: &ServerStore) -> Result<()>;
    async fn srv_delete(&mut self, server: &ServerStore) -> Result<()>;
}

impl StoreExt for Store {
    async fn init(&mut self) -> Result<()> {
        match self {
            Store::Json(store_ctx) => store_ctx.init().await,
            Store::Sqlite(store_ctx) => store_ctx.init().await,
        }
    }

    async fn settings_fetch(&self) -> Result<Settings> {
        match self {
            Store::Json(store_ctx) => store_ctx.settings_fetch().await,
            Store::Sqlite(store_ctx) => store_ctx.settings_fetch().await,
        }
    }

    async fn settings_save(&mut self, settings: &Settings) -> Result<()> {
        match self {
            Store::Json(store_ctx) => store_ctx.settings_save(settings).await,
            Store::Sqlite(store_ctx) => store_ctx.settings_save(settings).await,
        }
    }

    async fn srv_fetch_by_id(&self, id: &str) -> Result<Option<ServerStore>> {
        match self {
            Store::Json(store_ctx) => store_ctx.srv_fetch_by_id(id).await,
            Store::Sqlite(store_ctx) => store_ctx.srv_fetch_by_id(id).await,
        }
    }

    async fn srv_fetch_by_addr(&self, ip: &str, port: u16) -> Result<Option<ServerStore>> {
        match self {
            Store::Json(store_ctx) => store_ctx.srv_fetch_by_addr(ip, port).await,
            Store::Sqlite(store_ctx) => store_ctx.srv_fetch_by_addr(ip, port).await,
        }
    }

    async fn srv_fetch_all(&self) -> Result<Vec<ServerStore>> {
        match self {
            Store::Json(store_ctx) => store_ctx.srv_fetch_all().await,
            Store::Sqlite(store_ctx) => store_ctx.srv_fetch_all().await,
        }
    }

    async fn srv_add(&mut self, server: &ServerStore) -> Result<()> {
        match self {
            Store::Json(store_ctx) => store_ctx.srv_add(server).await,
            Store::Sqlite(store_ctx) => store_ctx.srv_add(server).await,
        }
    }

    async fn srv_update(&mut self, server: &ServerStore) -> Result<()> {
        match self {
            Store::Json(store_ctx) => store_ctx.srv_update(server).await,
            Store::Sqlite(store_ctx) => store_ctx.srv_update(server).await,
        }
    }

    async fn srv_delete(&mut self, server: &ServerStore) -> Result<()> {
        match self {
            Store::Json(store_ctx) => store_ctx.srv_delete(server).await,
            Store::Sqlite(store_ctx) => store_ctx.srv_delete(server).await,
        }
    }
}
