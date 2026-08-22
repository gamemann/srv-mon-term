/// Quake 3 engine query context (Call of Duty 1/2/4, Quake 3, Wolfenstein: ET, ...).
#[derive(Debug, Clone, Default)]
pub struct QueryQuake3Ctx {}

impl QueryQuake3Ctx {
    pub fn new() -> Self {
        Self::default()
    }
}

/// A single player entry from a `getstatus` reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quake3Player {
    pub name: String,
    pub score: i64,
    pub ping: u32,
}

/// Parsed `statusResponse` payload.
#[derive(Debug, Clone, Default)]
pub struct Quake3Status {
    pub vars: Vec<(String, String)>,
    pub players: Vec<Quake3Player>,
}

impl Quake3Status {
    pub fn var(&self, key: &str) -> Option<&str> {
        self.vars
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }
}
