pub mod json;
pub mod sqlite;

pub mod server;

use crate::store::{
    context::StoreCtx,
    types::{json::JsonStore, sqlite::SqliteStore},
};

pub enum Store {
    Json(StoreCtx<JsonStore>),
    Sqlite(StoreCtx<SqliteStore>),
}
