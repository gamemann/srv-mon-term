use a2s::players::Player;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServerUser {
    pub id: String,
    pub name: String,

    pub score: i64,
    pub duration: u64,
}

impl From<Player> for ServerUser {
    fn from(ply: Player) -> Self {
        ServerUser {
            id: ply.index.to_string(),
            name: ply.name,
            score: ply.score as i64,
            duration: ply.duration as u64,
        }
    }
}
