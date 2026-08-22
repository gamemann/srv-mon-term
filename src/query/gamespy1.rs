use std::time::Duration;

use anyhow::Result;
use tokio::time::Instant;

use crate::{
    query::{
        ext::{QueryAllResponse, QueryExt},
        proto::{
            net::UdpSession,
            text::{sanitize, strip_quake_colors},
        },
        types::{
            error::QueryError,
            ext::{InfoResponse, QueryResponse, UsersResponse, VarsResponse},
            gamespy::{GameSpyPlayer, GameSpyStatus, QueryGameSpy1Ctx},
        },
    },
    server::{data::ServerStatus, types::user::ServerUser, types::var::ServerVar},
};

const REQUEST: &[u8] = b"\\status\\";

/// Extra packets are collected with a short window since responses are commonly split.
const EXTRA_PACKET_WAIT: Duration = Duration::from_millis(250);
const MAX_PACKETS: usize = 8;

/// Parses the `\key\value\...` payload of a GameSpy v1 response.
pub fn parse_status(payload: &str) -> Result<GameSpyStatus, QueryError> {
    let mut fields = payload.split('\\').filter(|f| !f.is_empty());

    let mut vars: Vec<(String, String)> = Vec::new();
    let mut players: Vec<(usize, GameSpyPlayer)> = Vec::new();

    while let Some(key) = fields.next() {
        let key = key.trim().to_lowercase();

        // Packet bookkeeping keys carry no server information.
        if key == "final" || key == "queryid" {
            if key == "queryid" {
                fields.next();
            }

            continue;
        }

        let value = sanitize(&strip_quake_colors(fields.next().unwrap_or("")));

        match split_indexed(&key) {
            Some((field, idx)) => {
                let pos = match players.iter().position(|(i, _)| *i == idx) {
                    Some(pos) => pos,
                    None => {
                        players.push((idx, GameSpyPlayer::default()));

                        players.len() - 1
                    }
                };

                let entry = &mut players[pos].1;

                match field {
                    "player" | "playername" => entry.name = value,
                    "frags" | "score" => entry.score = value.parse().unwrap_or(0),
                    "ping" => entry.ping = value.parse().ok(),
                    "team" => entry.team = Some(value),
                    _ => {}
                }
            }
            None => vars.push((key, value)),
        }
    }

    if vars.is_empty() && players.is_empty() {
        return Err(QueryError::InvalidResponse(
            "empty GameSpy response".to_string(),
        ));
    }

    players.sort_by_key(|(idx, _)| *idx);

    Ok(GameSpyStatus {
        vars,
        players: players
            .into_iter()
            .map(|(_, p)| p)
            .filter(|p| !p.name.is_empty())
            .collect(),
    })
}

/// Splits keys like `player_3` into their field name and index.
fn split_indexed(key: &str) -> Option<(&str, usize)> {
    let (field, idx) = key.rsplit_once('_')?;

    Some((field, idx.parse().ok()?))
}

pub(crate) fn info_from_status(status: &GameSpyStatus) -> InfoResponse {
    let users_cnt = status
        .var("numplayers")
        .and_then(|v| v.parse().ok())
        .unwrap_or(status.players.len() as u16);

    let users_max = status
        .var("maxplayers")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let needs_pass = status
        .var("password")
        .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
        .unwrap_or(false);

    InfoResponse {
        srv_name: status.var("hostname").map(|v| v.to_string()),
        map_name: status
            .var("mapname")
            .or_else(|| status.var("maptitle"))
            .or_else(|| status.var("map"))
            .map(|v| v.to_string()),
        game_name: status
            .var("gamename")
            .or_else(|| status.var("game_id"))
            .or_else(|| status.var("gametype"))
            .map(|v| v.to_string()),
        game_port: status.var("hostport").and_then(|v| v.parse().ok()),
        game_dir: status.var("gamedir").map(|v| v.to_string()),
        game_id: None,
        users_cnt,
        users_max,
        bots_cnt: status.var("numbots").and_then(|v| v.parse().ok()),
        os: None,
        is_secure: false,
        is_dedicated: status.var("dedicated").map(|v| v != "0").unwrap_or(true),
        is_public: !needs_pass,
        version: status
            .var("gamever")
            .or_else(|| status.var("version"))
            .map(|v| v.to_string()),
    }
}

pub(crate) fn users_from_status(status: &GameSpyStatus) -> Vec<ServerUser> {
    status
        .players
        .iter()
        .enumerate()
        .map(|(idx, p)| ServerUser {
            id: idx.to_string(),
            name: p.name.clone(),
            score: p.score,
            duration: 0,
            ping: p.ping,
        })
        .collect()
}

pub(crate) fn vars_from_status(status: &GameSpyStatus) -> Vec<ServerVar> {
    status
        .vars
        .iter()
        .filter(|(k, v)| !k.is_empty() && !v.is_empty())
        .map(|(k, v)| ServerVar {
            name: k.clone(),
            value: v.clone(),
        })
        .collect()
}

impl QueryGameSpy1Ctx {
    async fn fetch(
        &self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<(GameSpyStatus, u64), QueryError> {
        let sess = UdpSession::connect(ip, port, timeout).await?;

        let start = Instant::now();

        let first = sess.request(REQUEST).await?;

        let latency = start.elapsed().as_micros() as u64;

        let mut payload = String::from_utf8_lossy(&first).to_string();

        // Responses larger than a datagram are split and terminated by a `\final\` marker.
        if !payload.contains("\\final\\") {
            for packet in sess.drain(EXTRA_PACKET_WAIT, MAX_PACKETS).await {
                payload.push_str(&String::from_utf8_lossy(&packet));

                if payload.contains("\\final\\") {
                    break;
                }
            }
        }

        Ok((parse_status(&payload)?, latency))
    }
}

impl QueryExt for QueryGameSpy1Ctx {
    async fn init(&mut self) -> Result<()> {
        Ok(())
    }

    async fn query_all(&mut self, ip: &str, port: u16, timeout: u64) -> Result<QueryAllResponse> {
        let (status, latency) = match self.fetch(ip, port, timeout).await {
            Ok(res) => res,
            Err(e) => return Ok(QueryAllResponse::from_status(e.status())),
        };

        Ok(QueryAllResponse {
            info: QueryResponse {
                status: ServerStatus::Online,
                latency,
                data: info_from_status(&status),
            },
            users: QueryResponse {
                status: ServerStatus::Online,
                latency,
                data: UsersResponse {
                    users: users_from_status(&status),
                },
            },
            vars: QueryResponse {
                status: ServerStatus::Online,
                latency,
                data: VarsResponse {
                    vars: vars_from_status(&status),
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
        Ok(self.query_all(ip, port, timeout).await?.info)
    }

    async fn query_users(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<UsersResponse>> {
        Ok(self.query_all(ip, port, timeout).await?.users)
    }

    async fn query_vars(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<VarsResponse>> {
        Ok(self.query_all(ip, port, timeout).await?.vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\\hostname\\Test Server\\gamename\\ut\\gamever\\436\\mapname\\DM-Deck16\\numplayers\\2\\maxplayers\\16\\hostport\\7777\\password\\0\\player_0\\Alice\\frags_0\\12\\ping_0\\40\\player_1\\Bob\\frags_1\\3\\ping_1\\80\\queryid\\1.1\\final\\";

    #[test]
    fn parses_status_payload() {
        let status = parse_status(SAMPLE).expect("failed to parse");

        assert_eq!(status.var("hostname"), Some("Test Server"));
        assert_eq!(status.players.len(), 2);
        assert_eq!(status.players[0].name, "Alice");
        assert_eq!(status.players[0].score, 12);
        assert_eq!(status.players[1].ping, Some(80));

        // Bookkeeping keys should not leak into the var list.
        assert!(status.var("queryid").is_none());
        assert!(status.var("final").is_none());
    }

    #[test]
    fn maps_info_fields() {
        let info = info_from_status(&parse_status(SAMPLE).unwrap());

        assert_eq!(info.srv_name.as_deref(), Some("Test Server"));
        assert_eq!(info.map_name.as_deref(), Some("DM-Deck16"));
        assert_eq!(info.users_cnt, 2);
        assert_eq!(info.users_max, 16);
        assert_eq!(info.game_port, Some(7777));
        assert!(info.is_public);
    }

    #[test]
    fn rejects_empty_payload() {
        assert!(parse_status("").is_err());
    }
}
