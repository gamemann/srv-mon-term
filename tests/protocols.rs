//! End to end tests for every query protocol.
//!
//! Each test spins up a local server that speaks just enough of the protocol to answer one
//! query, then runs the real client against it.

use std::net::SocketAddr;

use gmon::query::ext::QueryExt;
use gmon::query::types::Query;
use gmon::server::data::ServerStatus;
use gmon::server::types::query::ServerQueryType;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

const TIMEOUT: u64 = 2000;

/// Spawns a UDP responder that answers a single request with `build(request)`.
async fn udp_server<F>(build: F) -> SocketAddr
where
    F: Fn(&[u8]) -> Option<Vec<u8>> + Send + 'static,
{
    let sock = UdpSocket::bind("127.0.0.1:0").await.expect("bind failed");
    let addr = sock.local_addr().expect("no local addr");

    tokio::spawn(async move {
        let mut buf = vec![0u8; 4096];

        loop {
            let (len, peer) = match sock.recv_from(&mut buf).await {
                Ok(res) => res,
                Err(_) => return,
            };

            if let Some(reply) = build(&buf[..len]) {
                let _ = sock.send_to(&reply, peer).await;
            }
        }
    });

    addr
}

/// Spawns a TCP server that hands each accepted connection to `handle`.
async fn tcp_server<F, Fut>(handle: F) -> SocketAddr
where
    F: Fn(TcpStream) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind failed");
    let addr = listener.local_addr().expect("no local addr");

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    tokio::spawn(handle(stream));
                }
                Err(_) => return,
            }
        }
    });

    addr
}

async fn query_for(query_type: ServerQueryType) -> Query {
    Query::from_srv_type(&query_type)
        .await
        .expect("failed to build query")
}

#[tokio::test]
async fn queries_a_quake3_server() {
    let addr = udp_server(|req| {
        if !req.ends_with(b"getstatus\n") {
            return None;
        }

        Some(b"\xFF\xFF\xFF\xFFstatusResponse\n\\sv_maxclients\\32\\mapname\\mp_crash\\sv_hostname\\^1CoD4 ^7Server\\gamename\\CoD4MP\\shortversion\\1.7\\pswrd\\0\n7 55 \"Alice\"\n2 90 \"Bob\"\n".to_vec())
    })
    .await;

    let mut query = query_for(ServerQueryType::Quake3).await;

    let res = query
        .query_all("127.0.0.1", addr.port(), TIMEOUT)
        .await
        .expect("query failed");

    assert_eq!(res.info.status, ServerStatus::Online);
    assert_eq!(res.info.data.srv_name.as_deref(), Some("CoD4 Server"));
    assert_eq!(res.info.data.map_name.as_deref(), Some("mp_crash"));
    assert_eq!(res.info.data.users_cnt, 2);
    assert_eq!(res.info.data.users_max, 32);

    assert_eq!(res.users.data.users.len(), 2);
    assert_eq!(res.users.data.users[0].name, "Alice");
    assert_eq!(res.users.data.users[0].ping, Some(55));

    assert!(
        res.vars
            .data
            .vars
            .iter()
            .any(|v| v.name == "gamename" && v.value == "CoD4MP")
    );

    assert!(res.info.latency > 0);
}

#[tokio::test]
async fn queries_a_bedrock_server() {
    let addr = udp_server(|req| {
        if req.first() != Some(&0x01) {
            return None;
        }

        let motd = "MCPE;Bedrock Server;800;1.21.44;5;20;123456;World;Survival;1;19132;19133;";

        let mut out = vec![0x1C];

        out.extend_from_slice(&0i64.to_be_bytes());
        out.extend_from_slice(&7i64.to_be_bytes());
        out.extend_from_slice(&[
            0x00, 0xFF, 0xFF, 0x00, 0xFE, 0xFE, 0xFE, 0xFE, 0xFD, 0xFD, 0xFD, 0xFD, 0x12, 0x34,
            0x56, 0x78,
        ]);
        out.extend_from_slice(&(motd.len() as u16).to_be_bytes());
        out.extend_from_slice(motd.as_bytes());

        Some(out)
    })
    .await;

    let mut query = query_for(ServerQueryType::Bedrock).await;

    let res = query
        .query_all("127.0.0.1", addr.port(), TIMEOUT)
        .await
        .expect("query failed");

    assert_eq!(res.info.status, ServerStatus::Online);
    assert_eq!(res.info.data.srv_name.as_deref(), Some("Bedrock Server"));
    assert_eq!(res.info.data.users_cnt, 5);
    assert_eq!(res.info.data.users_max, 20);
    assert_eq!(res.info.data.map_name.as_deref(), Some("World"));
    assert!(res.vars.data.vars.iter().any(|v| v.name == "gamemode"));
}

#[tokio::test]
async fn queries_a_gamespy1_server() {
    let addr = udp_server(|req| {
        if req != b"\\status\\" {
            return None;
        }

        Some(b"\\hostname\\GS1 Server\\gamename\\ut\\mapname\\DM-Deck16\\numplayers\\1\\maxplayers\\16\\hostport\\7777\\player_0\\Alice\\frags_0\\9\\ping_0\\30\\final\\".to_vec())
    })
    .await;

    let mut query = query_for(ServerQueryType::GameSpy1).await;

    let res = query
        .query_all("127.0.0.1", addr.port(), TIMEOUT)
        .await
        .expect("query failed");

    assert_eq!(res.info.status, ServerStatus::Online);
    assert_eq!(res.info.data.srv_name.as_deref(), Some("GS1 Server"));
    assert_eq!(res.info.data.users_cnt, 1);
    assert_eq!(res.users.data.users[0].name, "Alice");
    assert_eq!(res.users.data.users[0].score, 9);
}

#[tokio::test]
async fn queries_a_gamespy3_server() {
    let addr = udp_server(|req| {
        // Challenge request.
        if req.len() == 7 && req[2] == 0x09 {
            let mut out = vec![0x09];

            out.extend_from_slice(&req[3..7]);
            out.extend_from_slice(b"9513307\0");

            return Some(out);
        }

        // Status request.
        if req.len() == 15 && req[2] == 0x00 {
            let mut out = vec![0x00];

            out.extend_from_slice(&req[3..7]);
            out.extend_from_slice(b"splitnum\0");
            out.extend_from_slice(&[0x80, 0x00]);

            for (key, value) in [
                ("hostname", "A Minecraft Server"),
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

            out.push(0);
            out.push(0x01);
            out.extend_from_slice(b"player_\0");
            out.push(0);
            out.extend_from_slice(b"Alice\0Bob\0");
            out.push(0);

            return Some(out);
        }

        None
    })
    .await;

    let mut query = query_for(ServerQueryType::GameSpy3).await;

    let res = query
        .query_all("127.0.0.1", addr.port(), TIMEOUT)
        .await
        .expect("query failed");

    assert_eq!(res.info.status, ServerStatus::Online);
    assert_eq!(
        res.info.data.srv_name.as_deref(),
        Some("A Minecraft Server")
    );
    assert_eq!(res.info.data.users_cnt, 2);
    assert_eq!(res.info.data.game_port, Some(25565));
    assert_eq!(res.users.data.users.len(), 2);
    assert_eq!(res.users.data.users[1].name, "Bob");
}

#[tokio::test]
async fn queries_a_minecraft_java_server() {
    const STATUS: &str = r#"{"version":{"name":"1.21.4","protocol":769},"players":{"max":20,"online":1,"sample":[{"name":"Alice","id":"069a79f4-44e9-4726-a5be-fca90e38aaf5"}]},"description":"§bJava Server"}"#;

    let addr = tcp_server(|mut stream| async move {
        // Read the handshake and the status request, both length prefixed.
        for _ in 0..2 {
            let mut len = [0u8; 1];

            if stream.read_exact(&mut len).await.is_err() {
                return;
            }

            let mut body = vec![0u8; len[0] as usize];

            if stream.read_exact(&mut body).await.is_err() {
                return;
            }
        }

        let mut packet = vec![0x00];

        // The JSON payload is prefixed with its length as a varint.
        let mut json_len = Vec::new();
        let mut value = STATUS.len();

        loop {
            if value & !0x7F == 0 {
                json_len.push(value as u8);

                break;
            }

            json_len.push(((value & 0x7F) as u8) | 0x80);
            value >>= 7;
        }

        packet.extend_from_slice(&json_len);
        packet.extend_from_slice(STATUS.as_bytes());

        let mut framed = Vec::new();
        let mut len = packet.len();

        loop {
            if len & !0x7F == 0 {
                framed.push(len as u8);

                break;
            }

            framed.push(((len & 0x7F) as u8) | 0x80);
            len >>= 7;
        }

        framed.extend_from_slice(&packet);

        let _ = stream.write_all(&framed).await;
    })
    .await;

    let mut query = query_for(ServerQueryType::Minecraft).await;

    let res = query
        .query_all("127.0.0.1", addr.port(), TIMEOUT)
        .await
        .expect("query failed");

    assert_eq!(res.info.status, ServerStatus::Online);
    assert_eq!(res.info.data.srv_name.as_deref(), Some("Java Server"));
    assert_eq!(res.info.data.version.as_deref(), Some("1.21.4"));
    assert_eq!(res.info.data.users_cnt, 1);
    assert_eq!(res.info.data.users_max, 20);
    assert_eq!(res.users.data.users[0].name, "Alice");
}

#[tokio::test]
async fn queries_a_fivem_server() {
    let addr = tcp_server(|mut stream| async move {
        let mut req = Vec::new();
        let mut chunk = [0u8; 1024];

        // Read until we have the full request head.
        loop {
            match stream.read(&mut chunk).await {
                Ok(0) => break,
                Ok(read) => {
                    req.extend_from_slice(&chunk[..read]);

                    if req.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                Err(_) => return,
            }
        }

        let head = String::from_utf8_lossy(&req).to_string();

        let body = if head.contains("/dynamic.json") {
            r#"{"hostname":"^2FiveM RP","clients":12,"sv_maxclients":"48","gametype":"Roleplay","mapname":"Los Santos"}"#
        } else if head.contains("/players.json") {
            r#"[{"id":1,"name":"Alice","ping":40},{"id":2,"name":"Bob","ping":-1}]"#
        } else {
            r#"{"version":99,"resources":["chat"],"vars":{"sv_enforceGameBuild":"2802","locale":"en-US"}}"#
        };

        let res = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );

        let _ = stream.write_all(res.as_bytes()).await;
    })
    .await;

    let mut query = query_for(ServerQueryType::FiveM).await;

    let res = query
        .query_all("127.0.0.1", addr.port(), TIMEOUT)
        .await
        .expect("query failed");

    assert_eq!(res.info.status, ServerStatus::Online);
    assert_eq!(res.info.data.srv_name.as_deref(), Some("FiveM RP"));
    assert_eq!(res.info.data.users_cnt, 12);
    assert_eq!(res.info.data.users_max, 48);

    assert_eq!(res.users.data.users.len(), 2);
    assert_eq!(res.users.data.users[0].ping, Some(40));
    assert_eq!(res.users.data.users[1].ping, None);

    assert!(res.vars.data.vars.iter().any(|v| v.name == "locale"));
}

#[tokio::test]
async fn reports_offline_when_nothing_answers() {
    // Nothing is listening on this socket, so the query has to time out.
    let sock = UdpSocket::bind("127.0.0.1:0").await.expect("bind failed");
    let port = sock.local_addr().expect("no local addr").port();

    drop(sock);

    let mut query = query_for(ServerQueryType::Quake3).await;

    let res = query
        .query_all("127.0.0.1", port, 250)
        .await
        .expect("query failed");

    assert_eq!(res.info.status, ServerStatus::Offline);
    assert_eq!(res.users.status, ServerStatus::Offline);
    assert_eq!(res.vars.status, ServerStatus::Offline);
}
