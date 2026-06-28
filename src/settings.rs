use std::collections::HashMap;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    #[serde(deserialize_with = "bool_from_str", default)]
    pub query_all_in_bg: bool,

    pub tui_draw_interval: u64,
    pub tui_input_poll_interval: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            query_all_in_bg: true,
            tui_draw_interval: 1000,
            tui_input_poll_interval: 100,
        }
    }
}

impl Settings {
    pub fn from_map(map: &HashMap<String, String>) -> Result<Self> {
        let json_map: serde_json::Map<String, Value> = map
            .into_iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect();

        let settings: Settings = serde_json::from_value(Value::Object(json_map))
            .map_err(|e| anyhow!("Failed to parse settings from map: {}", e))?;

        Ok(settings)
    }

    pub fn to_map(&self) -> HashMap<String, String> {
        let json_value = serde_json::to_value(self).unwrap_or(Value::Null);

        if let Value::Object(map) = json_value {
            map.into_iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                .collect()
        } else {
            HashMap::new()
        }
    }
}

fn bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "yes" | "true" | "1" => Ok(true),
        "no" | "false" | "0" => Ok(false),
        other => Err(serde::de::Error::custom(format!("invalid bool: {other}"))),
    }
}
