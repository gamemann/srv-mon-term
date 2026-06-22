use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServerQueryType {
    A2s,
}

impl Default for ServerQueryType {
    fn default() -> Self {
        ServerQueryType::A2s
    }
}

impl From<i32> for ServerQueryType {
    fn from(value: i32) -> Self {
        match value {
            0 => ServerQueryType::A2s,
            _ => ServerQueryType::default(),
        }
    }
}

impl From<ServerQueryType> for i32 {
    fn from(value: ServerQueryType) -> Self {
        match value {
            ServerQueryType::A2s => 0,
        }
    }
}

impl FromStr for ServerQueryType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "a2s" => Ok(ServerQueryType::A2s),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ServerQueryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerQueryType::A2s => write!(f, "A2S"),
        }
    }
}
