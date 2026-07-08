use serde::{Deserialize, Serialize};

use crate::{settings::Settings, store::server::ServerStore};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct JsonStateData {
    pub servers: Vec<ServerStore>,
    pub settings: Settings,
}
