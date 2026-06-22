use a2s::rules::Rule;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServerVar {
    pub name: String,
    pub value: String,
}

impl From<Rule> for ServerVar {
    fn from(rule: Rule) -> Self {
        ServerVar {
            name: rule.name.to_lowercase().trim().to_string(),
            value: rule.value,
        }
    }
}
