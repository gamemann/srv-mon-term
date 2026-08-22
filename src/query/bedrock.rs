use anyhow::Result;
use tokio::time::Instant;

use crate::{
    query::{
        ext::{QueryAllResponse, QueryExt},
        proto::{
            net::UdpSession,
            text::{sanitize, strip_minecraft_colors},
        },
        types::{
            error::QueryError,
            ext::{InfoResponse, QueryResponse, UsersResponse, VarsResponse},
            minecraft::{BedrockStatus, QueryBedrockCtx},
        },
    },
    server::{data::ServerStatus, types::var::ServerVar},
};

/// RakNet "offline message data ID" magic every unconnected packet carries.
const MAGIC: [u8; 16] = [
    0x00, 0xFF, 0xFF, 0x00, 0xFE, 0xFE, 0xFE, 0xFE, 0xFD, 0xFD, 0xFD, 0xFD, 0x12, 0x34, 0x56, 0x78,
];

const ID_UNCONNECTED_PING: u8 = 0x01;
const ID_UNCONNECTED_PONG: u8 = 0x1C;

fn ping_packet() -> Vec<u8> {
    let mut out = Vec::with_capacity(33);

    out.push(ID_UNCONNECTED_PING);

    // Timestamp; servers echo it back but we time the round trip ourselves.
    out.extend_from_slice(&0i64.to_be_bytes());
    out.extend_from_slice(&MAGIC);

    // Client GUID.
    out.extend_from_slice(&0i64.to_be_bytes());

    out
}

/// Extracts the semicolon separated MOTD payload out of an unconnected pong.
pub fn parse_pong(data: &[u8]) -> Result<BedrockStatus, QueryError> {
    if data.first() != Some(&ID_UNCONNECTED_PONG) {
        return Err(QueryError::InvalidResponse(
            "not an unconnected pong".to_string(),
        ));
    }

    // 1 byte id + 8 byte time + 8 byte server GUID + 16 byte magic + 2 byte string length.
    if data.len() < 35 {
        return Err(QueryError::InvalidResponse("pong too short".to_string()));
    }

    if data[17..33] != MAGIC {
        return Err(QueryError::InvalidResponse(
            "missing RakNet magic".to_string(),
        ));
    }

    let len = u16::from_be_bytes([data[33], data[34]]) as usize;
    let end = (35 + len).min(data.len());

    Ok(parse_motd(&String::from_utf8_lossy(&data[35..end])))
}

/// MOTD layout: `edition;motd;protocol;version;online;max;serverId;levelName;gamemode;...;portV4`
pub fn parse_motd(motd: &str) -> BedrockStatus {
    let fields: Vec<&str> = motd.split(';').collect();

    let field = |idx: usize| -> Option<String> {
        fields
            .get(idx)
            .map(|f| sanitize(&strip_minecraft_colors(f)))
            .filter(|f| !f.is_empty())
    };

    BedrockStatus {
        edition: field(0),
        motd: field(1),
        protocol: field(2),
        version: field(3),
        users_cnt: field(4).and_then(|v| v.parse().ok()).unwrap_or(0),
        users_max: field(5).and_then(|v| v.parse().ok()).unwrap_or(0),
        server_id: field(6),
        level_name: field(7),
        gamemode: field(8),
        port_v4: field(10).and_then(|v| v.parse().ok()),
    }
}

fn info_from_status(status: &BedrockStatus) -> InfoResponse {
    InfoResponse {
        srv_name: status.motd.clone(),
        map_name: status.level_name.clone(),
        game_name: Some(match status.edition.as_deref() {
            Some("MCPE") | None => "Minecraft (Bedrock)".to_string(),
            Some("MCEE") => "Minecraft (Education)".to_string(),
            Some(other) => format!("Minecraft ({})", other),
        }),
        game_port: status.port_v4,
        game_dir: None,
        game_id: None,
        users_cnt: status.users_cnt,
        users_max: status.users_max,
        bots_cnt: None,
        os: None,
        is_secure: false,
        is_dedicated: true,
        is_public: true,
        version: status.version.clone(),
    }
}

fn vars_from_status(status: &BedrockStatus) -> Vec<ServerVar> {
    let mut vars = Vec::new();

    let mut push = |name: &str, value: Option<String>| {
        if let Some(value) = value {
            vars.push(ServerVar {
                name: name.to_string(),
                value,
            });
        }
    };

    push("edition", status.edition.clone());
    push("protocol", status.protocol.clone());
    push("version", status.version.clone());
    push("level_name", status.level_name.clone());
    push("gamemode", status.gamemode.clone());
    push("server_id", status.server_id.clone());
    push("players_online", Some(status.users_cnt.to_string()));
    push("players_max", Some(status.users_max.to_string()));

    vars
}

impl QueryBedrockCtx {
    async fn fetch(
        &self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<(BedrockStatus, u64), QueryError> {
        let sess = UdpSession::connect(ip, port, timeout).await?;

        let start = Instant::now();

        let raw = sess.request(&ping_packet()).await?;

        let latency = start.elapsed().as_micros() as u64;

        Ok((parse_pong(&raw)?, latency))
    }
}

impl QueryExt for QueryBedrockCtx {
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
            // Bedrock's unconnected pong does not expose a player list.
            users: QueryResponse {
                status: ServerStatus::Online,
                latency,
                data: UsersResponse::default(),
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

    fn sample_pong() -> Vec<u8> {
        let motd = "MCPE;§aDedicated Server;800;1.21.44;3;10;13253860892328930865;Bedrock level;Survival;1;19132;19133;";

        let mut out = vec![ID_UNCONNECTED_PONG];

        out.extend_from_slice(&0i64.to_be_bytes());
        out.extend_from_slice(&1i64.to_be_bytes());
        out.extend_from_slice(&MAGIC);
        out.extend_from_slice(&(motd.len() as u16).to_be_bytes());
        out.extend_from_slice(motd.as_bytes());

        out
    }

    #[test]
    fn parses_pong() {
        let status = parse_pong(&sample_pong()).expect("failed to parse");

        assert_eq!(status.edition.as_deref(), Some("MCPE"));
        assert_eq!(status.motd.as_deref(), Some("Dedicated Server"));
        assert_eq!(status.version.as_deref(), Some("1.21.44"));
        assert_eq!(status.users_cnt, 3);
        assert_eq!(status.users_max, 10);
        assert_eq!(status.level_name.as_deref(), Some("Bedrock level"));
        assert_eq!(status.port_v4, Some(19132));
    }

    #[test]
    fn maps_info_fields() {
        let info = info_from_status(&parse_pong(&sample_pong()).unwrap());

        assert_eq!(info.game_name.as_deref(), Some("Minecraft (Bedrock)"));
        assert_eq!(info.map_name.as_deref(), Some("Bedrock level"));
        assert_eq!(info.game_port, Some(19132));
    }

    #[test]
    fn rejects_bad_packets() {
        assert!(parse_pong(&[0x00; 40]).is_err());
        assert!(parse_pong(&[ID_UNCONNECTED_PONG]).is_err());
    }
}
