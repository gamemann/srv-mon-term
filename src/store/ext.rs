use anyhow::Result;

use crate::{server::types::Server, settings::Settings, store::types::Store};

#[allow(async_fn_in_trait)]
pub trait StoreExt {
    async fn init(&mut self) -> Result<()>;

    async fn settings_fetch(&mut self) -> Result<Settings>;
    async fn settings_save(&mut self, settings: &Settings) -> Result<()>;

    async fn srv_fetch_by_id(&mut self, id: &str) -> Result<Option<Server>>;
    async fn srv_fetch_by_addr(&mut self, ip: &str, port: u16) -> Result<Option<Server>>;
    async fn srv_fetch_all(&mut self) -> Result<Vec<Server>>;

    async fn srv_add(&mut self, server: &Server) -> Result<()>;
    async fn srv_update(&mut self, server: &Server) -> Result<()>;
    async fn srv_delete(&mut self, server: &Server) -> Result<()>;
}

impl StoreExt for Store {
    async fn init(&mut self) -> Result<()> {
        match self {
            Store::Json(store_ctx) => store_ctx.init().await,
            Store::Sqlite(store_ctx) => store_ctx.init().await,
        }
    }

    async fn settings_fetch(&mut self) -> Result<Settings> {
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

    async fn srv_fetch_by_id(&mut self, id: &str) -> Result<Option<Server>> {
        match self {
            Store::Json(store_ctx) => store_ctx.srv_fetch_by_id(id).await,
            Store::Sqlite(store_ctx) => store_ctx.srv_fetch_by_id(id).await,
        }
    }

    async fn srv_fetch_by_addr(&mut self, ip: &str, port: u16) -> Result<Option<Server>> {
        match self {
            Store::Json(store_ctx) => store_ctx.srv_fetch_by_addr(ip, port).await,
            Store::Sqlite(store_ctx) => store_ctx.srv_fetch_by_addr(ip, port).await,
        }
    }

    async fn srv_fetch_all(&mut self) -> Result<Vec<Server>> {
        match self {
            Store::Json(store_ctx) => store_ctx.srv_fetch_all().await,
            Store::Sqlite(store_ctx) => store_ctx.srv_fetch_all().await,
        }
    }

    async fn srv_add(&mut self, server: &Server) -> Result<()> {
        match self {
            Store::Json(store_ctx) => store_ctx.srv_add(server).await,
            Store::Sqlite(store_ctx) => store_ctx.srv_add(server).await,
        }
    }

    async fn srv_update(&mut self, server: &Server) -> Result<()> {
        match self {
            Store::Json(store_ctx) => store_ctx.srv_update(server).await,
            Store::Sqlite(store_ctx) => store_ctx.srv_update(server).await,
        }
    }

    async fn srv_delete(&mut self, server: &Server) -> Result<()> {
        match self {
            Store::Json(store_ctx) => store_ctx.srv_delete(server).await,
            Store::Sqlite(store_ctx) => store_ctx.srv_delete(server).await,
        }
    }
}
