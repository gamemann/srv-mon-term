use serde::{Deserialize, Serialize};

use crate::{settings::Settings, store::server::ServerStore};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct JsonStoreFormat {
    pub settings: Settings,
    pub servers: Vec<ServerStore>,
}
