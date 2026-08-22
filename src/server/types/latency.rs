use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::Display;

/// How latency is measured for a server.
///
/// The `Self*` variants reuse the timing of the regular query task, while the query variants
/// send an extra request using whichever protocol the server is configured with.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize, Display, Default)]
pub enum ServerLatencyType {
    #[default]
    SelfInfo,
    SelfUsers,
    SelfVars,

    #[serde(alias = "QueryInfo")]
    A2sInfo,
    #[serde(alias = "QueryUsers")]
    A2sPlayers,
    #[serde(alias = "QueryVars")]
    A2sRules,

    Icmp,
}

impl ServerLatencyType {
    pub const ALL: [ServerLatencyType; 7] = [
        ServerLatencyType::SelfInfo,
        ServerLatencyType::SelfUsers,
        ServerLatencyType::SelfVars,
        ServerLatencyType::A2sInfo,
        ServerLatencyType::A2sPlayers,
        ServerLatencyType::A2sRules,
        ServerLatencyType::Icmp,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ServerLatencyType::SelfInfo => "self-info",
            ServerLatencyType::SelfUsers => "self-users",
            ServerLatencyType::SelfVars => "self-vars",
            ServerLatencyType::A2sInfo => "query-info",
            ServerLatencyType::A2sPlayers => "query-users",
            ServerLatencyType::A2sRules => "query-vars",
            ServerLatencyType::Icmp => "icmp",
        }
    }
}

impl FromStr for ServerLatencyType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let needle = s.trim().to_lowercase().replace(['_', ' '], "-");

        ServerLatencyType::ALL
            .into_iter()
            .find(|t| t.name() == needle || t.to_string().to_lowercase() == needle)
            .ok_or(())
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerLatency {
    pub online: bool,
    pub type_: ServerLatencyType,
    pub ts: u64,
    pub val: u64, // Stored in microseconds!
}

impl ServerLatency {
    /// Retrieves the amount of latency in milliseconds.
    ///
    /// # Returns
    ///
    /// The latency in milliseconds as a floating-point number.
    pub fn get_latency_ms(&self) -> f64 {
        return self.val as f64 / 1000.0;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_latency_types() {
        assert_eq!("icmp".parse(), Ok(ServerLatencyType::Icmp));
        assert_eq!("self_info".parse(), Ok(ServerLatencyType::SelfInfo));
        assert_eq!("query-users".parse(), Ok(ServerLatencyType::A2sPlayers));
        assert_eq!("A2sRules".parse(), Ok(ServerLatencyType::A2sRules));
        assert_eq!("nope".parse::<ServerLatencyType>(), Err(()));
    }

    #[test]
    fn round_trips_store_ids() {
        for latency_type in ServerLatencyType::ALL {
            let id: i32 = latency_type.into();

            assert_eq!(ServerLatencyType::from(id), latency_type);
        }
    }

    #[test]
    fn converts_micros_to_ms() {
        let latency = ServerLatency {
            val: 1500,
            ..Default::default()
        };

        assert_eq!(latency.get_latency_ms(), 1.5);
    }
}
