use std::collections::HashMap;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{context::Context, log_info, logger::Logger, logger::level::LogLevel};

pub const SETTINGS_DEFAULT_TUI_DRAW_INTERVAL: u64 = 500;
pub const SETTINGS_DEFAULT_TUI_INPUT_POLL_INTERVAL: u64 = 500;

pub const SETTINGS_DEFAULT_LOG_MAX_BUFFER_SIZE: usize = 250;
pub const SETTINGS_DEFAULT_LOG_PATH: Option<String> = None;
pub const SETTINGS_DEFAULT_LOG_LEVELS: &[LogLevel] = &[
    LogLevel::Info,
    LogLevel::Warn,
    LogLevel::Error,
    LogLevel::Fatal,
];

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Settings {
    pub tui_draw_interval: u64,
    pub tui_input_poll_interval: u64,
    pub log_path: Option<String>,
    pub log_max_buffer_size: usize,
    pub log_levels: Vec<LogLevel>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            tui_draw_interval: SETTINGS_DEFAULT_TUI_DRAW_INTERVAL,
            tui_input_poll_interval: SETTINGS_DEFAULT_TUI_INPUT_POLL_INTERVAL,
            log_path: SETTINGS_DEFAULT_LOG_PATH,
            log_max_buffer_size: SETTINGS_DEFAULT_LOG_MAX_BUFFER_SIZE,
            log_levels: SETTINGS_DEFAULT_LOG_LEVELS.to_vec(),
        }
    }
}

impl Settings {
    pub fn from_map(map: &HashMap<String, String>) -> Result<Self> {
        let mut json_map = serde_json::Map::new();

        for (k, v) in map {
            let parsed_value =
                serde_json::from_str::<Value>(v).unwrap_or_else(|_| Value::String(v.clone()));

            json_map.insert(k.clone(), parsed_value);
        }

        let settings: Settings = serde_json::from_value(Value::Object(json_map))
            .map_err(|e| anyhow!("Failed to parse settings from map: {}", e))?;

        Ok(settings)
    }

    pub fn to_map(&self) -> HashMap<String, String> {
        let json_value = serde_json::to_value(self).unwrap_or(Value::Null);

        if let Value::Object(map) = json_value {
            map.into_iter()
                .map(|(k, v)| {
                    let string_val = match v {
                        Value::String(s) => s,
                        _ => v.to_string(),
                    };
                    (k, string_val)
                })
                .collect()
        } else {
            HashMap::new()
        }
    }

    pub async fn log_settings(ctx: Context) {
        log_info!(ctx, "Current log settings:");

        let settings = ctx.settings.read().await;
        log_info!(ctx, "  tui_draw_interval: {}", settings.tui_draw_interval);
        log_info!(
            ctx,
            "  tui_input_poll_interval: {}",
            settings.tui_input_poll_interval
        );
        log_info!(ctx, "  log_path: {:?}", settings.log_path);
        log_info!(
            ctx,
            "  log_max_buffer_size: {}",
            settings.log_max_buffer_size
        );
        log_info!(ctx, "  log_levels: {:?}", settings.log_levels);
    }
}

fn _bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
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
