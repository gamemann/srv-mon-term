use serde::{Deserialize, Serialize};

use crate::server::{
    Server,
    types::{
        DEFAULT_LATENCY_HISTORY_SIZE, DEFAULT_QUERY_INTERVAL, DEFAULT_QUERY_TIMEOUT,
        latency::ServerLatencyType, query::ServerQueryType,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for ServerStore {
    fn default() -> Self {
        // Mirrors `Server::default()` so a stored server without explicit settings behaves the
        // same as one created at runtime.
        let server = Server::default();

        ServerStore {
            id: String::new(),
            ip: server.ip,
            port: server.port,
            port_query: server.port_query,
            display_name: server.display_name,
            query_interval: DEFAULT_QUERY_INTERVAL,
            query_timeout: DEFAULT_QUERY_TIMEOUT,
            query_type: server.query_type,
            latency_interval: server.latency_interval,
            latency_timeout: server.latency_timeout,
            latency_type: server.latency_type,
            latency_history_size: DEFAULT_LATENCY_HISTORY_SIZE,
        }
    }
}

impl ServerStore {
    pub fn to_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
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
        // Zeroed intervals would spin the scheduler, so fall back to the defaults.
        let defaults = Server::default();

        Server {
            ip: store.ip,
            port: store.port,
            port_query: store.port_query,
            display_name: store.display_name,
            query_interval: if store.query_interval > 0 {
                store.query_interval
            } else {
                defaults.query_interval
            },
            query_timeout: if store.query_timeout > 0 {
                store.query_timeout
            } else {
                defaults.query_timeout
            },
            query_type: store.query_type,
            latency_interval: store.latency_interval.filter(|v| *v > 0),
            latency_timeout: store.latency_timeout.filter(|v| *v > 0),
            latency_type: store.latency_type,
            latency_history_size: if store.latency_history_size > 0 {
                store.latency_history_size
            } else {
                defaults.latency_history_size
            },
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_runtime_defaults() {
        let store = ServerStore::default();
        let server = Server::default();

        assert_eq!(store.query_interval, server.query_interval);
        assert_eq!(store.query_timeout, server.query_timeout);
        assert_eq!(store.latency_history_size, server.latency_history_size);
    }

    #[test]
    fn zeroed_values_fall_back_to_defaults() {
        let store = ServerStore {
            query_interval: 0,
            query_timeout: 0,
            latency_history_size: 0,
            latency_interval: Some(0),
            ..Default::default()
        };

        let server: Server = store.into();
        let defaults = Server::default();

        assert_eq!(server.query_interval, defaults.query_interval);
        assert_eq!(server.query_timeout, defaults.query_timeout);
        assert_eq!(server.latency_history_size, defaults.latency_history_size);
        assert_eq!(server.latency_interval, None);
    }

    #[test]
    fn round_trips_settings() {
        let store = ServerStore {
            ip: "127.0.0.1".to_string(),
            port: 27015,
            port_query: Some(27016),
            query_type: ServerQueryType::Quake3,
            query_interval: 5000,
            ..Default::default()
        };

        let server: Server = store.clone().into();

        assert_eq!(server.query_type, ServerQueryType::Quake3);
        assert_eq!(server.query_port(), 27016);
        assert_eq!(server.query_interval, 5000);

        let back: ServerStore = server.into();

        assert_eq!(back.port, store.port);
        assert_eq!(back.port_query, store.port_query);
        assert_eq!(back.query_type, store.query_type);
    }
}
