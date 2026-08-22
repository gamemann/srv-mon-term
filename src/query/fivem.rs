use anyhow::Result;
use serde_json::Value;
use tokio::time::Instant;

use crate::{
    query::{
        ext::{QueryAllResponse, QueryExt},
        proto::{
            net::http_get,
            text::{sanitize, strip_quake_colors},
        },
        types::{
            error::QueryError,
            ext::{InfoResponse, QueryResponse, UsersResponse, VarsResponse},
            fivem::QueryFiveMCtx,
        },
    },
    server::{data::ServerStatus, types::user::ServerUser, types::var::ServerVar},
};

/// Server metadata (resources, vars, version).
const PATH_INFO: &str = "/info.json";
/// Live player list.
const PATH_PLAYERS: &str = "/players.json";
/// Hostname, player counts, map and game type.
const PATH_DYNAMIC: &str = "/dynamic.json";

fn value_to_string(value: &Value) -> Option<String> {
    let raw = match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => return None,
    };

    let out = sanitize(&strip_quake_colors(&raw));

    if out.is_empty() { None } else { Some(out) }
}

fn json_str(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(value_to_string)
}

fn json_u16(value: &Value, key: &str) -> Option<u16> {
    json_str(value, key).and_then(|v| v.parse().ok())
}

/// Builds the info response out of the `dynamic.json` and `info.json` documents.
pub fn parse_info(dynamic: &Value, info: &Value) -> InfoResponse {
    let vars = info.get("vars").cloned().unwrap_or(Value::Null);

    InfoResponse {
        srv_name: json_str(dynamic, "hostname").or_else(|| json_str(&vars, "sv_projectName")),
        map_name: json_str(dynamic, "mapname").or_else(|| json_str(&vars, "mapname")),
        game_name: json_str(dynamic, "gametype")
            .or_else(|| json_str(&vars, "gamename"))
            .or_else(|| Some("FiveM".to_string())),
        game_port: None,
        game_dir: None,
        game_id: None,
        users_cnt: json_u16(dynamic, "clients").unwrap_or(0),
        users_max: json_u16(dynamic, "sv_maxclients")
            .or_else(|| json_u16(&vars, "sv_maxClients"))
            .unwrap_or(0),
        bots_cnt: None,
        os: None,
        is_secure: json_str(&vars, "sv_scriptHookAllowed")
            .map(|v| v == "false")
            .unwrap_or(false),
        is_dedicated: true,
        is_public: json_str(&vars, "sv_lan")
            .map(|v| v != "true")
            .unwrap_or(true),
        version: json_str(&vars, "sv_enforceGameBuild")
            .or_else(|| json_str(info, "version"))
            .or_else(|| json_str(&vars, "gamebuild")),
    }
}

/// Converts the `players.json` array into our user list.
pub fn parse_users(players: &Value) -> Vec<ServerUser> {
    players
        .as_array()
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let name = json_str(entry, "name")?;

                    Some(ServerUser {
                        id: json_str(entry, "id").unwrap_or_default(),
                        name,
                        score: 0,
                        duration: 0,
                        // FiveM reports -1 for players whose ping is not known yet.
                        ping: entry
                            .get("ping")
                            .and_then(|p| p.as_i64())
                            .filter(|p| *p >= 0)
                            .map(|p| p as u32),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Flattens the `vars` object of `info.json` into our var list.
pub fn parse_vars(info: &Value) -> Vec<ServerVar> {
    let mut vars: Vec<ServerVar> = info
        .get("vars")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(key, value)| {
                    Some(ServerVar {
                        name: key.trim().to_lowercase(),
                        value: value_to_string(value)?,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    if let Some(resources) = info.get("resources").and_then(|r| r.as_array()) {
        vars.push(ServerVar {
            name: "resources".to_string(),
            value: resources.len().to_string(),
        });
    }

    if let Some(version) = json_str(info, "version") {
        vars.push(ServerVar {
            name: "version".to_string(),
            value: version,
        });
    }

    vars
}

impl QueryFiveMCtx {
    async fn fetch_json(
        &self,
        ip: &str,
        port: u16,
        path: &str,
        timeout: u64,
    ) -> Result<Value, QueryError> {
        let body = http_get(ip, port, path, timeout).await?;

        serde_json::from_str(&body)
            .map_err(|e| QueryError::InvalidResponse(format!("bad JSON from {}: {}", path, e)))
    }
}

impl QueryExt for QueryFiveMCtx {
    async fn init(&mut self) -> Result<()> {
        Ok(())
    }

    async fn query_all(&mut self, ip: &str, port: u16, timeout: u64) -> Result<QueryAllResponse> {
        let start = Instant::now();

        let dynamic = match self.fetch_json(ip, port, PATH_DYNAMIC, timeout).await {
            Ok(res) => res,
            Err(e) => return Ok(QueryAllResponse::from_status(e.status())),
        };

        let latency = start.elapsed().as_micros() as u64;

        // The remaining endpoints are optional; a server that answers /dynamic.json is online.
        let info = self
            .fetch_json(ip, port, PATH_INFO, timeout)
            .await
            .unwrap_or(Value::Null);

        let players = self
            .fetch_json(ip, port, PATH_PLAYERS, timeout)
            .await
            .unwrap_or(Value::Null);

        Ok(QueryAllResponse {
            info: QueryResponse {
                status: ServerStatus::Online,
                latency,
                data: parse_info(&dynamic, &info),
            },
            users: QueryResponse {
                status: ServerStatus::Online,
                latency,
                data: UsersResponse {
                    users: parse_users(&players),
                },
            },
            vars: QueryResponse {
                status: ServerStatus::Online,
                latency,
                data: VarsResponse {
                    vars: parse_vars(&info),
                },
            },
        })
    }

    async fn query_info(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<InfoResponse>> {
        let start = Instant::now();

        let dynamic = match self.fetch_json(ip, port, PATH_DYNAMIC, timeout).await {
            Ok(res) => res,
            Err(e) => {
                return Ok(QueryResponse::<InfoResponse> {
                    status: e.status(),
                    ..Default::default()
                });
            }
        };

        let latency = start.elapsed().as_micros() as u64;

        let info = self
            .fetch_json(ip, port, PATH_INFO, timeout)
            .await
            .unwrap_or(Value::Null);

        Ok(QueryResponse {
            status: ServerStatus::Online,
            latency,
            data: parse_info(&dynamic, &info),
        })
    }

    async fn query_users(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<UsersResponse>> {
        let start = Instant::now();

        let players = match self.fetch_json(ip, port, PATH_PLAYERS, timeout).await {
            Ok(res) => res,
            Err(e) => {
                return Ok(QueryResponse::<UsersResponse> {
                    status: e.status(),
                    ..Default::default()
                });
            }
        };

        Ok(QueryResponse {
            status: ServerStatus::Online,
            latency: start.elapsed().as_micros() as u64,
            data: UsersResponse {
                users: parse_users(&players),
            },
        })
    }

    async fn query_vars(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<VarsResponse>> {
        let start = Instant::now();

        let info = match self.fetch_json(ip, port, PATH_INFO, timeout).await {
            Ok(res) => res,
            Err(e) => {
                return Ok(QueryResponse::<VarsResponse> {
                    status: e.status(),
                    ..Default::default()
                });
            }
        };

        Ok(QueryResponse {
            status: ServerStatus::Online,
            latency: start.elapsed().as_micros() as u64,
            data: VarsResponse {
                vars: parse_vars(&info),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dynamic() -> Value {
        serde_json::json!({
            "hostname": "^2Cool ^7Roleplay",
            "clients": 32,
            "sv_maxclients": "64",
            "gametype": "Roleplay",
            "mapname": "Los Santos"
        })
    }

    fn info() -> Value {
        serde_json::json!({
            "version": 1234,
            "resources": ["spawnmanager", "chat", "mapmanager"],
            "vars": {
                "sv_projectName": "Cool RP",
                "sv_enforceGameBuild": "2802",
                "sv_lan": "false",
                "locale": "en-US"
            }
        })
    }

    #[test]
    fn maps_info_fields() {
        let parsed = parse_info(&dynamic(), &info());

        assert_eq!(parsed.srv_name.as_deref(), Some("Cool Roleplay"));
        assert_eq!(parsed.users_cnt, 32);
        assert_eq!(parsed.users_max, 64);
        assert_eq!(parsed.map_name.as_deref(), Some("Los Santos"));
        assert_eq!(parsed.version.as_deref(), Some("2802"));
        assert!(parsed.is_public);
    }

    #[test]
    fn maps_players() {
        let players = serde_json::json!([
            {"id": 1, "name": "Alice", "ping": 42},
            {"id": 2, "name": "Bob", "ping": -1}
        ]);

        let users = parse_users(&players);

        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "Alice");
        assert_eq!(users[0].ping, Some(42));
        assert_eq!(users[1].ping, None);
    }

    #[test]
    fn maps_vars() {
        let vars = parse_vars(&info());

        assert!(
            vars.iter()
                .any(|v| v.name == "locale" && v.value == "en-US")
        );
        assert!(vars.iter().any(|v| v.name == "resources" && v.value == "3"));
    }
}
