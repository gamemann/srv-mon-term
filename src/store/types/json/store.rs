use serde::{Deserialize, Serialize};

use crate::{server::types::Server, settings::Settings};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsonStoreFormat {
    pub settings: Settings,
    pub servers: Vec<Server>,
}
