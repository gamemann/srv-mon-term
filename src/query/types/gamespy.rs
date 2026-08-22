/// GameSpy v1 query context (`\status\`), used by Unreal Engine 1/2 era titles.
#[derive(Debug, Clone, Default)]
pub struct QueryGameSpy1Ctx {}

impl QueryGameSpy1Ctx {
    pub fn new() -> Self {
        Self::default()
    }
}

/// GameSpy v3 query context, also spoken by Minecraft's optional query listener.
#[derive(Debug, Clone, Default)]
pub struct QueryGameSpy3Ctx {}

impl QueryGameSpy3Ctx {
    pub fn new() -> Self {
        Self::default()
    }
}

/// A player parsed out of a GameSpy response.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GameSpyPlayer {
    pub name: String,
    pub score: i64,
    pub ping: Option<u32>,
    pub team: Option<String>,
}

/// Parsed GameSpy status payload.
#[derive(Debug, Clone, Default)]
pub struct GameSpyStatus {
    pub vars: Vec<(String, String)>,
    pub players: Vec<GameSpyPlayer>,
}

impl GameSpyStatus {
    pub fn var(&self, key: &str) -> Option<&str> {
        self.vars
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }
}
