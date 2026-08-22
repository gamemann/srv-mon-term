use std::time::Duration;

use anyhow::Result;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Instant, timeout};

use crate::{
    query::{
        ext::{QueryAllResponse, QueryExt},
        proto::{
            net::tcp_connect,
            text::{sanitize, strip_minecraft_colors},
            varint::{read_varint, write_string, write_varint},
        },
        types::{
            error::QueryError,
            ext::{InfoResponse, QueryResponse, UsersResponse, VarsResponse},
            minecraft::QueryMinecraftCtx,
        },
    },
    server::{data::ServerStatus, types::user::ServerUser, types::var::ServerVar},
};

/// Protocol version sent during the handshake. -1 means "undefined", which every modern
/// server accepts for a status request.
const PROTOCOL_VERSION: i32 = -1;

/// Guards against a malicious or broken server announcing an absurd packet length.
const MAX_PACKET_LEN: usize = 4 * 1024 * 1024;

fn framed(payload: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 5);

    write_varint(payload.len() as i32, &mut out);
    out.extend_from_slice(&payload);

    out
}

fn handshake_packet(host: &str, port: u16) -> Vec<u8> {
    let mut packet = Vec::new();

    // Packet ID 0x00 (handshake).
    write_varint(0x00, &mut packet);
    write_varint(PROTOCOL_VERSION, &mut packet);
    write_string(host, &mut packet);

    packet.extend_from_slice(&port.to_be_bytes());

    // Next state: 1 (status).
    write_varint(1, &mut packet);

    framed(packet)
}

fn status_request_packet() -> Vec<u8> {
    let mut packet = Vec::new();

    write_varint(0x00, &mut packet);

    framed(packet)
}

async fn read_varint_stream(stream: &mut TcpStream) -> Result<i32, QueryError> {
    let mut buf = Vec::with_capacity(5);

    loop {
        let mut byte = [0u8; 1];

        stream.read_exact(&mut byte).await?;

        buf.push(byte[0]);

        match read_varint(&buf) {
            Ok((value, _)) => return Ok(value),
            Err(_) if buf.len() < 5 => continue,
            Err(e) => return Err(e),
        }
    }
}

/// Reads a single length-prefixed status packet and returns its JSON payload.
async fn read_status_packet(stream: &mut TcpStream) -> Result<String, QueryError> {
    let len = read_varint_stream(stream).await?;

    if len <= 0 || len as usize > MAX_PACKET_LEN {
        return Err(QueryError::InvalidResponse(format!(
            "bad packet length {}",
            len
        )));
    }

    let mut body = vec![0u8; len as usize];
    stream.read_exact(&mut body).await?;

    let (packet_id, mut offset) = read_varint(&body)?;

    if packet_id != 0x00 {
        return Err(QueryError::InvalidResponse(format!(
            "unexpected packet id {}",
            packet_id
        )));
    }

    let (str_len, read) = read_varint(&body[offset..])?;
    offset += read;

    let end = offset
        .checked_add(str_len.max(0) as usize)
        .filter(|end| *end <= body.len())
        .ok_or_else(|| QueryError::InvalidResponse("truncated status payload".to_string()))?;

    Ok(String::from_utf8_lossy(&body[offset..end]).to_string())
}

/// Flattens a chat component (string, array or object) into plain text.
pub fn chat_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Array(items) => items.iter().map(chat_to_string).collect(),
        Value::Object(obj) => {
            let mut out = obj
                .get("text")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();

            if out.is_empty()
                && let Some(translate) = obj.get("translate").and_then(|t| t.as_str())
            {
                out.push_str(translate);
            }

            if let Some(extra) = obj.get("extra") {
                out.push_str(&chat_to_string(extra));
            }

            out
        }
        _ => String::new(),
    }
}

/// Builds the info/users/vars responses from a Server List Ping JSON document.
pub fn parse_status(
    json: &str,
) -> Result<(InfoResponse, Vec<ServerUser>, Vec<ServerVar>), QueryError> {
    let root: Value = serde_json::from_str(json)
        .map_err(|e| QueryError::InvalidResponse(format!("bad status JSON: {}", e)))?;

    let players = root.get("players");

    let users_cnt = players
        .and_then(|p| p.get("online"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u16;

    let users_max = players
        .and_then(|p| p.get("max"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u16;

    let motd = root
        .get("description")
        .map(chat_to_string)
        .map(|d| sanitize(&strip_minecraft_colors(&d)))
        .filter(|d| !d.is_empty());

    let version = root
        .get("version")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .map(|v| sanitize(&strip_minecraft_colors(v)));

    let protocol = root
        .get("version")
        .and_then(|v| v.get("protocol"))
        .and_then(|v| v.as_i64());

    let info = InfoResponse {
        srv_name: motd,
        map_name: None,
        game_name: Some("Minecraft".to_string()),
        game_port: None,
        game_dir: None,
        game_id: None,
        users_cnt,
        users_max,
        bots_cnt: None,
        os: None,
        is_secure: root
            .get("enforcesSecureChat")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        is_dedicated: true,
        is_public: true,
        version,
    };

    let users = players
        .and_then(|p| p.get("sample"))
        .and_then(|s| s.as_array())
        .map(|sample| {
            sample
                .iter()
                .filter_map(|entry| {
                    let name = entry.get("name")?.as_str()?;

                    Some(ServerUser {
                        id: entry
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        name: sanitize(&strip_minecraft_colors(name)),
                        score: 0,
                        duration: 0,
                        ping: None,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut vars = Vec::new();

    if let Some(protocol) = protocol {
        vars.push(ServerVar {
            name: "protocol".to_string(),
            value: protocol.to_string(),
        });
    }

    if let Some(secure) = root.get("enforcesSecureChat").and_then(|v| v.as_bool()) {
        vars.push(ServerVar {
            name: "enforces_secure_chat".to_string(),
            value: secure.to_string(),
        });
    }

    if let Some(mods) = root
        .get("modinfo")
        .and_then(|m| m.get("modList"))
        .and_then(|m| m.as_array())
    {
        vars.push(ServerVar {
            name: "mods".to_string(),
            value: mods.len().to_string(),
        });
    }

    if let Some(mod_type) = root
        .get("modinfo")
        .and_then(|m| m.get("type"))
        .and_then(|v| v.as_str())
    {
        vars.push(ServerVar {
            name: "mod_type".to_string(),
            value: mod_type.to_string(),
        });
    }

    vars.push(ServerVar {
        name: "players_online".to_string(),
        value: users_cnt.to_string(),
    });

    vars.push(ServerVar {
        name: "players_max".to_string(),
        value: users_max.to_string(),
    });

    Ok((info, users, vars))
}

impl QueryMinecraftCtx {
    async fn fetch(
        &self,
        ip: &str,
        port: u16,
        timeout_ms: u64,
    ) -> Result<(String, u64), QueryError> {
        let mut stream = tcp_connect(ip, port, timeout_ms).await?;

        let dur = Duration::from_millis(timeout_ms.max(1));

        let start = Instant::now();

        let exchange = async {
            stream.write_all(&handshake_packet(ip, port)).await?;
            stream.write_all(&status_request_packet()).await?;

            read_status_packet(&mut stream).await
        };

        let json = match timeout(dur, exchange).await {
            Ok(res) => res?,
            Err(_) => return Err(QueryError::Timeout),
        };

        Ok((json, start.elapsed().as_micros() as u64))
    }
}

impl QueryExt for QueryMinecraftCtx {
    async fn init(&mut self) -> Result<()> {
        Ok(())
    }

    async fn query_all(&mut self, ip: &str, port: u16, timeout: u64) -> Result<QueryAllResponse> {
        let (json, latency) = match self.fetch(ip, port, timeout).await {
            Ok(res) => res,
            Err(e) => return Ok(QueryAllResponse::from_status(e.status())),
        };

        let (info, users, vars) = match parse_status(&json) {
            Ok(res) => res,
            Err(e) => return Ok(QueryAllResponse::from_status(e.status())),
        };

        Ok(QueryAllResponse {
            info: QueryResponse {
                status: ServerStatus::Online,
                latency,
                data: info,
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

    const SAMPLE: &str = r#"{
        "version": {"name": "1.21.4", "protocol": 769},
        "players": {"max": 100, "online": 2, "sample": [
            {"name": "§aNotch", "id": "069a79f4-44e9-4726-a5be-fca90e38aaf5"},
            {"name": "Jeb", "id": "853c80ef-3c37-49fd-aa49-938b674adae6"}
        ]},
        "description": {"text": "A ", "extra": [{"text": "§cMinecraft §rServer"}]},
        "enforcesSecureChat": true
    }"#;

    #[test]
    fn parses_status_json() {
        let (info, users, vars) = parse_status(SAMPLE).expect("failed to parse");

        assert_eq!(info.srv_name.as_deref(), Some("A Minecraft Server"));
        assert_eq!(info.version.as_deref(), Some("1.21.4"));
        assert_eq!(info.users_cnt, 2);
        assert_eq!(info.users_max, 100);
        assert!(info.is_secure);

        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "Notch");

        assert!(
            vars.iter()
                .any(|v| v.name == "protocol" && v.value == "769")
        );
    }

    #[test]
    fn handles_plain_description() {
        let (info, _, _) = parse_status(r#"{"description": "Hello", "players": {}}"#).unwrap();

        assert_eq!(info.srv_name.as_deref(), Some("Hello"));
        assert_eq!(info.users_cnt, 0);
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(parse_status("not json").is_err());
    }

    #[test]
    fn builds_expected_handshake() {
        let packet = handshake_packet("localhost", 25565);

        // Length prefix, packet id, then the -1 protocol version as a 5 byte varint.
        assert_eq!(packet[1], 0x00);
        assert_eq!(&packet[2..7], &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        assert!(packet.ends_with(&[0x63, 0xDD, 0x01]));
    }
}
