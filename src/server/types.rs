pub mod data;
pub mod latency;
pub mod query;
pub mod tasks;

pub mod user;
pub mod var;

use serde::{Deserialize, Serialize};

use crate::server::types::{data::ServerData, latency::ServerLatencyType, query::ServerQueryType};

pub const DEFAULT_QUERY_INTERVAL: u64 = 1000;
pub const DEFAULT_QUERY_TIMEOUT: u64 = 2000;
pub const DEFAULT_LATENCY_HISTORY_SIZE: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct Server {
    pub ip: String,
    pub port: u16,

    pub display_name: Option<String>,

    pub port_query: Option<u16>,

    pub query_interval: u64,
    pub query_timeout: u64,
    pub query_type: ServerQueryType,

    pub latency_interval: Option<u64>,
    pub latency_timeout: Option<u64>,
    pub latency_type: ServerLatencyType,
    pub latency_history_size: usize,

    #[serde(skip)]
    pub data: ServerData,
}

impl Server {
    /// The port we actually send queries to.
    ///
    /// `port` is the port players connect to, which is not always the port that answers
    /// queries (Source servers, Minecraft's query listener, ...).
    pub fn query_port(&self) -> u16 {
        self.port_query.unwrap_or(self.port)
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            ip: String::new(),
            port: 0,

            display_name: None,

            port_query: None,

            query_interval: DEFAULT_QUERY_INTERVAL,
            query_timeout: DEFAULT_QUERY_TIMEOUT,
            query_type: ServerQueryType::default(),

            latency_interval: None,
            latency_timeout: None,
            latency_type: ServerLatencyType::default(),
            latency_history_size: DEFAULT_LATENCY_HISTORY_SIZE,

            data: ServerData::default(),
        }
    }
}
