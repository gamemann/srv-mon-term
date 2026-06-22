use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize, Display)]
pub enum ServerLatencyType {
    SelfInfo,
    SelfUsers,
    SelfVars,

    A2sInfo,
    A2sPlayers,
    A2sRules,
    Icmp,
}

impl Default for ServerLatencyType {
    fn default() -> Self {
        ServerLatencyType::SelfInfo
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerLatency {
    pub online: bool,
    pub type_: ServerLatencyType,
    pub ts: u64,
    pub val: u64,
}

impl Default for ServerLatency {
    fn default() -> Self {
        Self {
            online: false,
            type_: ServerLatencyType::SelfInfo,
            ts: 0,
            val: 0,
        }
    }
}

impl From<i32> for ServerLatencyType {
    fn from(value: i32) -> Self {
        match value {
            0 => ServerLatencyType::SelfInfo,
            1 => ServerLatencyType::SelfUsers,
            2 => ServerLatencyType::SelfVars,
            3 => ServerLatencyType::A2sInfo,
            4 => ServerLatencyType::A2sPlayers,
            5 => ServerLatencyType::A2sRules,
            6 => ServerLatencyType::Icmp,
            _ => ServerLatencyType::SelfInfo,
        }
    }
}

impl From<ServerLatencyType> for i32 {
    fn from(value: ServerLatencyType) -> Self {
        match value {
            ServerLatencyType::SelfInfo => 0,
            ServerLatencyType::SelfUsers => 1,
            ServerLatencyType::SelfVars => 2,
            ServerLatencyType::A2sInfo => 3,
            ServerLatencyType::A2sPlayers => 4,
            ServerLatencyType::A2sRules => 5,
            ServerLatencyType::Icmp => 6,
        }
    }
}
