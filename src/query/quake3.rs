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
            quake3::{Quake3Player, Quake3Status, QueryQuake3Ctx},
        },
    },
    server::{data::ServerStatus, types::user::ServerUser, types::var::ServerVar},
};

const HEADER: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
const REQUEST: &[u8] = b"\xFF\xFF\xFF\xFFgetstatus\n";

/// Parses a Quake 3 `statusResponse` / `infoResponse` payload.
pub fn parse_status(data: &[u8]) -> Result<Quake3Status, QueryError> {
    let body = data
        .strip_prefix(&HEADER)
        .ok_or_else(|| QueryError::InvalidResponse("missing Quake 3 header".to_string()))?;

    let text = String::from_utf8_lossy(body);

    let mut parts = text.splitn(2, '\n');

    let header = parts.next().unwrap_or("").trim();

    if !header.eq_ignore_ascii_case("statusResponse")
        && !header.eq_ignore_ascii_case("infoResponse")
    {
        return Err(QueryError::InvalidResponse(format!(
            "unexpected response type '{}'",
            header
        )));
    }

    let rest = parts.next().unwrap_or("");

    let mut lines = rest.split('\n');

    let vars = parse_vars(lines.next().unwrap_or(""));

    let players = lines
        .filter(|l| !l.trim().is_empty())
        .filter_map(parse_player)
        .collect();

    Ok(Quake3Status { vars, players })
}

fn parse_vars(line: &str) -> Vec<(String, String)> {
    let mut fields = line.split('\\').filter(|f| !f.is_empty());

    let mut out = Vec::new();

    while let Some(key) = fields.next() {
        let value = fields.next().unwrap_or("");

        out.push((
            key.trim().to_lowercase(),
            sanitize(&strip_quake_colors(value)),
        ));
    }

    out
}

/// Player lines look like `score ping "name"`, where the name may contain spaces.
fn parse_player(line: &str) -> Option<Quake3Player> {
    let line = line.trim();

    let mut fields = line.splitn(3, ' ');

    let score = fields.next()?.parse::<i64>().ok()?;
    let ping = fields.next()?.parse::<u32>().ok()?;

    let name = fields.next().unwrap_or("").trim();
    let name = name.strip_prefix('"').unwrap_or(name);
    let name = name.strip_suffix('"').unwrap_or(name);

    Some(Quake3Player {
        name: sanitize(&strip_quake_colors(name)),
        score,
        ping,
    })
}

fn info_from_status(status: &Quake3Status) -> InfoResponse {
    let users_max = status
        .var("sv_maxclients")
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(0);

    // `clients` is only present in getinfo replies, so fall back to the player list length.
    let users_cnt = status
        .var("clients")
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(status.players.len() as u16);

    let bots_cnt = status
        .var("bots")
        .and_then(|v| v.parse::<u16>().ok())
        .or_else(|| {
            if status.players.is_empty() {
                None
            } else {
                Some(status.players.iter().filter(|p| p.ping == 0).count() as u16)
            }
        });

    let needs_pass = status
        .var("pswrd")
        .or_else(|| status.var("g_needpass"))
        .or_else(|| status.var("password"))
        .map(|v| v != "0")
        .unwrap_or(false);

    InfoResponse {
        srv_name: status
            .var("sv_hostname")
            .or_else(|| status.var("hostname"))
            .map(|v| v.to_string()),
        map_name: status.var("mapname").map(|v| v.to_string()),
        game_name: status
            .var("gamename")
            .or_else(|| status.var("gametype"))
            .or_else(|| status.var("g_gametype"))
            .map(|v| v.to_string()),
        game_port: None,
        game_dir: status
            .var("fs_game")
            .or_else(|| status.var("gamedir"))
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string()),
        game_id: None,
        users_cnt,
        users_max,
        bots_cnt,
        os: None,
        is_secure: status
            .var("sv_punkbuster")
            .map(|v| v != "0")
            .unwrap_or(false),
        is_dedicated: status.var("dedicated").map(|v| v != "0").unwrap_or(true),
        is_public: !needs_pass,
        version: status
            .var("shortversion")
            .or_else(|| status.var("version"))
            .or_else(|| status.var("protocol"))
            .map(|v| v.to_string()),
    }
}

impl QueryQuake3Ctx {
    async fn fetch(
        &self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<(Quake3Status, u64), QueryError> {
        let sess = UdpSession::connect(ip, port, timeout).await?;

        let start = Instant::now();

        let raw = sess.request(REQUEST).await?;

        let latency = start.elapsed().as_micros() as u64;

        Ok((parse_status(&raw)?, latency))
    }
}

impl QueryExt for QueryQuake3Ctx {
    async fn init(&mut self) -> Result<()> {
        Ok(())
    }

    async fn query_all(&mut self, ip: &str, port: u16, timeout: u64) -> Result<QueryAllResponse> {
        let (status, latency) = match self.fetch(ip, port, timeout).await {
            Ok(res) => res,
            Err(e) => return Ok(QueryAllResponse::from_status(e.status())),
        };

        let users = status
            .players
            .iter()
            .enumerate()
            .map(|(idx, p)| ServerUser {
                id: idx.to_string(),
                name: p.name.clone(),
                score: p.score,
                duration: 0,
                ping: Some(p.ping),
            })
            .collect();

        let vars = status
            .vars
            .iter()
            .filter(|(k, v)| !k.is_empty() && !v.is_empty())
            .map(|(k, v)| ServerVar {
                name: k.clone(),
                value: v.clone(),
            })
            .collect();

        Ok(QueryAllResponse {
            info: QueryResponse {
                status: ServerStatus::Online,
                latency,
                data: info_from_status(&status),
            },
            users: QueryResponse {
                status: ServerStatus::Online,
                latency,
                data: UsersResponse { users },
            },
            vars: QueryResponse {
                status: ServerStatus::Online,
                latency,
                data: VarsResponse { vars },
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

    const SAMPLE: &[u8] = b"\xFF\xFF\xFF\xFFstatusResponse\n\\sv_maxclients\\32\\mapname\\mp_crash\\sv_hostname\\^1Test ^7Server\\gamename\\CoD4MP\\shortversion\\1.7\\pswrd\\0\n5 42 \"Player One\"\n0 0 \"A Bot\"\n";

    #[test]
    fn parses_status_response() {
        let status = parse_status(SAMPLE).expect("failed to parse");

        assert_eq!(status.var("mapname"), Some("mp_crash"));
        assert_eq!(status.var("sv_hostname"), Some("Test Server"));
        assert_eq!(status.players.len(), 2);

        assert_eq!(
            status.players[0],
            Quake3Player {
                name: "Player One".to_string(),
                score: 5,
                ping: 42,
            }
        );
    }

    #[test]
    fn builds_info_from_status() {
        let info = info_from_status(&parse_status(SAMPLE).unwrap());

        assert_eq!(info.srv_name.as_deref(), Some("Test Server"));
        assert_eq!(info.users_cnt, 2);
        assert_eq!(info.users_max, 32);
        assert_eq!(info.bots_cnt, Some(1));
        assert_eq!(info.version.as_deref(), Some("1.7"));
        assert!(info.is_public);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_status(b"nonsense").is_err());
        assert!(parse_status(b"\xFF\xFF\xFF\xFFgetchallenge\n").is_err());
    }
}
