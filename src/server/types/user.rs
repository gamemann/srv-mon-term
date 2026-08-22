use a2s::players::Player;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerUser {
    pub id: String,
    pub name: String,

    pub score: i64,

    /// Connection time in seconds. Not every protocol reports it.
    pub duration: u64,

    /// Round trip time to the server in milliseconds, when the protocol reports it.
    pub ping: Option<u32>,
}

impl Default for ServerUser {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            score: 0,
            duration: 0,
            ping: None,
        }
    }
}

impl From<Player> for ServerUser {
    fn from(ply: Player) -> Self {
        ServerUser {
            id: ply.index.to_string(),
            name: ply.name,
            score: ply.score as i64,
            duration: ply.duration as u64,
            ping: None,
        }
    }
}
