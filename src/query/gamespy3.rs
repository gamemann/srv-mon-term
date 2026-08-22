use std::time::Duration;

use anyhow::Result;
use tokio::time::Instant;

use crate::{
    query::{
        ext::{QueryAllResponse, QueryExt},
        gamespy1::{info_from_status, users_from_status, vars_from_status},
        proto::{
            net::UdpSession,
            text::{sanitize, strip_quake_colors},
        },
        types::{
            error::QueryError,
            ext::{InfoResponse, QueryResponse, UsersResponse, VarsResponse},
            gamespy::{GameSpyPlayer, GameSpyStatus, QueryGameSpy3Ctx},
        },
    },
    server::data::ServerStatus,
};

const MAGIC: [u8; 2] = [0xFE, 0xFD];
const TYPE_CHALLENGE: u8 = 0x09;
const TYPE_STATUS: u8 = 0x00;

/// Arbitrary session id; the low bits are the only ones some servers echo back.
const SESSION_ID: [u8; 4] = [0x00, 0x00, 0x00, 0x01];

/// Requesting all four sections is what makes servers include the player list.
const FULL_STAT_PADDING: [u8; 4] = [0xFF, 0xFF, 0xFF, 0x01];

const EXTRA_PACKET_WAIT: Duration = Duration::from_millis(250);
const MAX_PACKETS: usize = 8;

fn challenge_packet() -> Vec<u8> {
    let mut out = Vec::with_capacity(7);

    out.extend_from_slice(&MAGIC);
    out.push(TYPE_CHALLENGE);
    out.extend_from_slice(&SESSION_ID);

    out
}

fn status_packet(challenge: i32) -> Vec<u8> {
    let mut out = Vec::with_capacity(15);

    out.extend_from_slice(&MAGIC);
    out.push(TYPE_STATUS);
    out.extend_from_slice(&SESSION_ID);
    out.extend_from_slice(&challenge.to_be_bytes());
    out.extend_from_slice(&FULL_STAT_PADDING);

    out
}

/// Pulls the ASCII challenge token out of a `0x09` reply.
pub fn parse_challenge(data: &[u8]) -> Result<i32, QueryError> {
    if data.first() != Some(&TYPE_CHALLENGE) || data.len() < 6 {
        return Err(QueryError::Challenge(
            "malformed challenge reply".to_string(),
        ));
    }

    let token = String::from_utf8_lossy(&data[5..])
        .trim_end_matches('\0')
        .trim()
        .to_string();

    token
        .parse::<i32>()
        .map_err(|_| QueryError::Challenge(format!("bad challenge token '{}'", token)))
}

/// Reads a null terminated string starting at `cursor`, advancing past the terminator.
fn read_cstr(data: &[u8], cursor: &mut usize) -> String {
    let start = *cursor;

    while *cursor < data.len() && data[*cursor] != 0 {
        *cursor += 1;
    }

    let out = String::from_utf8_lossy(&data[start..*cursor]).to_string();

    // Step over the terminator when we found one.
    if *cursor < data.len() {
        *cursor += 1;
    }

    out
}

/// Parses a full-stat response body (header already stripped).
pub fn parse_full_stat(data: &[u8]) -> Result<GameSpyStatus, QueryError> {
    if data.first() != Some(&TYPE_STATUS) || data.len() < 5 {
        return Err(QueryError::InvalidResponse(
            "malformed status reply".to_string(),
        ));
    }

    let mut cursor = 5;

    // Optional split marker: "splitnum\0" followed by the packet index and a flag byte.
    if data[cursor..].starts_with(b"splitnum\0") {
        cursor += 9;
        cursor = (cursor + 2).min(data.len());
    }

    let mut vars = Vec::new();

    loop {
        let key = read_cstr(data, &mut cursor);

        if key.is_empty() {
            break;
        }

        let value = read_cstr(data, &mut cursor);

        vars.push((
            key.trim().to_lowercase(),
            sanitize(&strip_quake_colors(&value)),
        ));
    }

    // Player/team sections: a marker byte, the field name, then one value per player.
    let mut fields: Vec<(String, Vec<String>)> = Vec::new();

    while cursor < data.len() {
        // Section marker (0x01 for players, 0x02 for teams, ...).
        cursor += 1;

        let field = read_cstr(data, &mut cursor);

        if field.is_empty() {
            break;
        }

        // Field lists are separated from their values by an empty string.
        let _ = read_cstr(data, &mut cursor);

        let mut values = Vec::new();

        loop {
            let value = read_cstr(data, &mut cursor);

            if value.is_empty() {
                break;
            }

            values.push(sanitize(&strip_quake_colors(&value)));
        }

        fields.push((field.trim().to_lowercase(), values));
    }

    let find = |name: &str| -> Option<&Vec<String>> {
        fields
            .iter()
            .find(|(f, _)| f.trim_end_matches('_') == name)
            .map(|(_, v)| v)
    };

    let names = find("player").cloned().unwrap_or_default();
    let scores = find("score").cloned().unwrap_or_default();
    let pings = find("ping").cloned().unwrap_or_default();
    let teams = find("team").cloned().unwrap_or_default();

    let players = names
        .iter()
        .enumerate()
        .filter(|(_, name)| !name.is_empty())
        .map(|(idx, name)| GameSpyPlayer {
            name: name.clone(),
            score: scores.get(idx).and_then(|s| s.parse().ok()).unwrap_or(0),
            ping: pings.get(idx).and_then(|p| p.parse().ok()),
            team: teams.get(idx).cloned(),
        })
        .collect();

    if vars.is_empty() {
        return Err(QueryError::InvalidResponse(
            "no server variables in response".to_string(),
        ));
    }

    Ok(GameSpyStatus { vars, players })
}

impl QueryGameSpy3Ctx {
    async fn fetch(
        &self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<(GameSpyStatus, u64), QueryError> {
        let sess = UdpSession::connect(ip, port, timeout).await?;

        let start = Instant::now();

        let challenge = parse_challenge(&sess.request(&challenge_packet()).await?)?;

        let first = sess.request(&status_packet(challenge)).await?;

        let latency = start.elapsed().as_micros() as u64;

        let mut payload = first;

        // Long responses are split across datagrams that each repeat the 5 byte header.
        if payload.len() >= 1400 {
            for packet in sess.drain(EXTRA_PACKET_WAIT, MAX_PACKETS).await {
                if packet.len() > 5 {
                    payload.extend_from_slice(&packet[5..]);
                }
            }
        }

        Ok((parse_full_stat(&payload)?, latency))
    }
}

impl QueryExt for QueryGameSpy3Ctx {
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

    fn sample_response() -> Vec<u8> {
        let mut out = vec![TYPE_STATUS];

        out.extend_from_slice(&SESSION_ID);
        out.extend_from_slice(b"splitnum\0");
        out.extend_from_slice(&[0x80, 0x00]);

        for (key, value) in [
            ("hostname", "A Minecraft Server"),
            ("gametype", "SMP"),
            ("game_id", "MINECRAFT"),
            ("version", "1.21.4"),
            ("map", "world"),
            ("numplayers", "2"),
            ("maxplayers", "20"),
            ("hostport", "25565"),
        ] {
            out.extend_from_slice(key.as_bytes());
            out.push(0);
            out.extend_from_slice(value.as_bytes());
            out.push(0);
        }

        // End of the key/value section.
        out.push(0);

        out.push(0x01);
        out.extend_from_slice(b"player_\0");
        out.push(0);
        out.extend_from_slice(b"Alice\0");
        out.extend_from_slice(b"Bob\0");
        out.push(0);

        out
    }

    #[test]
    fn parses_full_stat() {
        let status = parse_full_stat(&sample_response()).expect("failed to parse");

        assert_eq!(status.var("hostname"), Some("A Minecraft Server"));
        assert_eq!(status.var("map"), Some("world"));
        assert_eq!(status.players.len(), 2);
        assert_eq!(status.players[1].name, "Bob");
    }

    #[test]
    fn maps_info_fields() {
        let info = info_from_status(&parse_full_stat(&sample_response()).unwrap());

        assert_eq!(info.srv_name.as_deref(), Some("A Minecraft Server"));
        assert_eq!(info.users_cnt, 2);
        assert_eq!(info.users_max, 20);
        assert_eq!(info.game_port, Some(25565));
    }

    #[test]
    fn parses_challenge_token() {
        let mut reply = vec![TYPE_CHALLENGE];
        reply.extend_from_slice(&SESSION_ID);
        reply.extend_from_slice(b"-1234567\0");

        assert_eq!(parse_challenge(&reply).unwrap(), -1234567);
        assert!(parse_challenge(&[TYPE_CHALLENGE]).is_err());
    }

    #[test]
    fn rejects_bad_status() {
        assert!(parse_full_stat(&[0x09, 0, 0, 0, 1]).is_err());
    }
}
