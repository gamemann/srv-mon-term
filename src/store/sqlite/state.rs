use std::sync::{Arc, Mutex};

use rusqlite::Connection;

#[derive(Debug, Default)]
pub struct StoreSqliteState {
    pub conn: Option<Arc<Mutex<Connection>>>,
}
