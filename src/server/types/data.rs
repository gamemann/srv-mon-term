use serde::{Deserialize, Serialize};
use strum::Display;

use crate::server::types::{user::ServerUser, var::ServerVar};

use a2s::info::ServerOS as A2sServerOs;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
pub enum ServerStatus {
    Unknown,
    Online,
    Offline,
    Error(u16),
}

impl Default for ServerStatus {
    fn default() -> Self {
        ServerStatus::Unknown
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
pub enum ServerOs {
    Linux,
    Windows,
    Mac,
}

impl From<A2sServerOs> for ServerOs {
    fn from(os: A2sServerOs) -> Self {
        match os {
            A2sServerOs::Linux => ServerOs::Linux,
            A2sServerOs::Windows => ServerOs::Windows,
            A2sServerOs::Mac => ServerOs::Mac,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServerData {
    pub users: Vec<ServerUser>,
    pub vars: Vec<ServerVar>,

    pub users_cur: u16,
    pub users_max: u16,
    pub bots_cur: Option<u16>,

    pub srv_name: Option<String>,
    pub game_name: Option<String>,
    pub map_name: Option<String>,

    pub game_id: Option<u16>,
    pub game_dir: Option<String>,

    pub os: Option<ServerOs>,

    pub is_secure: bool,
    pub is_dedicated: bool,
    pub is_public: bool,

    pub version: Option<String>,

    pub last_updated: Option<u64>,
}

impl Default for ServerData {
    fn default() -> Self {
        Self {
            users: Vec::new(),
            vars: Vec::new(),

            users_cur: 0,
            users_max: 0,
            bots_cur: None,

            srv_name: None,
            game_name: None,
            game_dir: None,
            map_name: None,
            game_id: None,

            os: None,

            is_secure: false,
            is_dedicated: false,
            is_public: false,

            version: None,

            last_updated: None,
        }
    }
}
