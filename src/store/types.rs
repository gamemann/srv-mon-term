pub mod server;

use crate::store::{
    context::StoreCtx,
    json::{base::JsonStore, opts::StoreJsonOpts, state::StoreJsonState},
    sqlite::{base::SqliteStore, opts::StoreSqliteOpts, state::StoreSqliteState},
};

pub enum Store {
    Json(StoreCtx<JsonStore, StoreJsonState, StoreJsonOpts>),
    Sqlite(StoreCtx<SqliteStore, StoreSqliteState, StoreSqliteOpts>),
}
