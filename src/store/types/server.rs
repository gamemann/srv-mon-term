use serde::{Deserialize, Serialize};

use crate::server::{
    Server,
    types::{latency::ServerLatencyType, query::ServerQueryType},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ServerStore {
    pub id: String,

    pub ip: String,
    pub port: u16,

    // Opts.
    pub port_query: Option<u16>,

    pub display_name: Option<String>,

    pub query_interval: u64,
    pub query_timeout: u64,
    pub query_type: ServerQueryType,

    pub latency_interval: Option<u64>,
    pub latency_timeout: Option<u64>,
    pub latency_type: ServerLatencyType,
    pub latency_history_size: usize,
}

impl From<Server> for ServerStore {
    fn from(server: Server) -> Self {
        ServerStore {
            id: String::new(),
            ip: server.ip,
            port: server.port,
            port_query: server.port_query,
            display_name: server.display_name,
            query_interval: server.query_interval,
            query_timeout: server.query_timeout,
            query_type: server.query_type,
            latency_interval: server.latency_interval,
            latency_timeout: server.latency_timeout,
            latency_type: server.latency_type,
            latency_history_size: server.latency_history_size,
        }
    }
}

impl From<ServerStore> for Server {
    fn from(store: ServerStore) -> Self {
        Server {
            ip: store.ip,
            port: store.port,
            port_query: store.port_query,
            display_name: store.display_name,
            query_interval: store.query_interval,
            query_timeout: store.query_timeout,
            query_type: store.query_type,
            latency_interval: store.latency_interval,
            latency_timeout: store.latency_timeout,
            latency_type: store.latency_type,
            latency_history_size: store.latency_history_size,
            ..Default::default()
        }
    }
}
