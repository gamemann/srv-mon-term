use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket, lookup_host};
use tokio::time::timeout;

use crate::query::types::error::QueryError;

/// Maximum amount of bytes we're willing to read out of a single datagram or HTTP response.
pub const MAX_DATAGRAM_SIZE: usize = 65535;
pub const MAX_HTTP_SIZE: usize = 2 * 1024 * 1024;

/// Resolves `host:port` into a single socket address.
pub async fn resolve(host: &str, port: u16) -> Result<SocketAddr, QueryError> {
    // Fast path for plain IP literals so we avoid a resolver round trip.
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(SocketAddr::new(ip, port));
    }

    let mut addrs = lookup_host((host, port))
        .await
        .map_err(|e| QueryError::BadAddress(format!("{}: {}", host, e)))?;

    addrs
        .next()
        .ok_or_else(|| QueryError::BadAddress(format!("no addresses found for {}", host)))
}

fn bind_addr_for(addr: &SocketAddr) -> &'static str {
    if addr.is_ipv6() {
        "[::]:0"
    } else {
        "0.0.0.0:0"
    }
}

/// A connected UDP socket with a fixed receive timeout.
///
/// Protocols that need a challenge exchange re-use the same session so the server sees a
/// consistent source port.
pub struct UdpSession {
    sock: UdpSocket,
    timeout: Duration,
}

impl UdpSession {
    pub async fn connect(host: &str, port: u16, timeout_ms: u64) -> Result<Self, QueryError> {
        let addr = resolve(host, port).await?;

        let sock = UdpSocket::bind(bind_addr_for(&addr)).await?;
        sock.connect(addr).await?;

        Ok(Self {
            sock,
            timeout: Duration::from_millis(timeout_ms.max(1)),
        })
    }

    pub async fn send(&self, data: &[u8]) -> Result<(), QueryError> {
        self.sock.send(data).await?;

        Ok(())
    }

    pub async fn recv(&self) -> Result<Vec<u8>, QueryError> {
        self.recv_timeout(self.timeout).await
    }

    pub async fn recv_timeout(&self, dur: Duration) -> Result<Vec<u8>, QueryError> {
        let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];

        let len = match timeout(dur, self.sock.recv(&mut buf)).await {
            Ok(res) => res?,
            Err(_) => return Err(QueryError::Timeout),
        };

        buf.truncate(len);

        Ok(buf)
    }

    /// Sends a payload and waits for a single reply.
    pub async fn request(&self, data: &[u8]) -> Result<Vec<u8>, QueryError> {
        self.send(data).await?;

        self.recv().await
    }

    /// Reads any remaining datagrams that arrive within `dur`, used by protocols that split
    /// large responses across multiple packets.
    pub async fn drain(&self, dur: Duration, max_packets: usize) -> Vec<Vec<u8>> {
        let mut out = Vec::new();

        while out.len() < max_packets {
            match self.recv_timeout(dur).await {
                Ok(packet) => out.push(packet),
                Err(_) => break,
            }
        }

        out
    }
}

/// Opens a TCP connection with a connect timeout applied.
pub async fn tcp_connect(host: &str, port: u16, timeout_ms: u64) -> Result<TcpStream, QueryError> {
    let addr = resolve(host, port).await?;

    let dur = Duration::from_millis(timeout_ms.max(1));

    let stream = match timeout(dur, TcpStream::connect(addr)).await {
        Ok(res) => res?,
        Err(_) => return Err(QueryError::Timeout),
    };

    // Game protocols are request/response, so batching small writes only adds latency.
    let _ = stream.set_nodelay(true);

    Ok(stream)
}

/// Performs a minimal HTTP/1.1 GET request and returns the response body.
///
/// Only used for servers that expose their query data over plain HTTP (e.g. FiveM).
pub async fn http_get(
    host: &str,
    port: u16,
    path: &str,
    timeout_ms: u64,
) -> Result<String, QueryError> {
    let dur = Duration::from_millis(timeout_ms.max(1));

    let mut stream = tcp_connect(host, port, timeout_ms).await?;

    let req = format!(
        "GET {} HTTP/1.1\r\nHost: {}:{}\r\nUser-Agent: gmon\r\nAccept: application/json\r\nConnection: close\r\n\r\n",
        path, host, port
    );

    match timeout(dur, stream.write_all(req.as_bytes())).await {
        Ok(res) => res?,
        Err(_) => return Err(QueryError::Timeout),
    }

    let mut buf = Vec::new();

    match timeout(dur, async {
        let mut chunk = [0u8; 8192];

        loop {
            let read = stream.read(&mut chunk).await?;

            if read == 0 {
                break;
            }

            buf.extend_from_slice(&chunk[..read]);

            if buf.len() >= MAX_HTTP_SIZE {
                break;
            }
        }

        Ok::<(), std::io::Error>(())
    })
    .await
    {
        Ok(res) => res?,
        Err(_) => return Err(QueryError::Timeout),
    }

    parse_http_response(&buf)
}

/// Splits an HTTP response into its status line, headers and body, returning the body.
pub fn parse_http_response(raw: &[u8]) -> Result<String, QueryError> {
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| QueryError::InvalidResponse("malformed HTTP response".to_string()))?;

    let head = String::from_utf8_lossy(&raw[..split]);

    let status = head
        .lines()
        .next()
        .ok_or_else(|| QueryError::InvalidResponse("missing HTTP status line".to_string()))?;

    let code = status
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse::<u16>().ok())
        .ok_or_else(|| QueryError::InvalidResponse(format!("bad HTTP status: {}", status)))?;

    if !(200..300).contains(&code) {
        return Err(QueryError::InvalidResponse(format!("HTTP status {}", code)));
    }

    let body = &raw[split + 4..];

    let is_chunked = head.to_lowercase().contains("transfer-encoding: chunked");

    let body = if is_chunked {
        decode_chunked(body)?
    } else {
        body.to_vec()
    };

    Ok(String::from_utf8_lossy(&body).to_string())
}

fn decode_chunked(mut body: &[u8]) -> Result<Vec<u8>, QueryError> {
    let mut out = Vec::new();

    loop {
        let line_end = body
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or_else(|| QueryError::InvalidResponse("malformed chunk header".to_string()))?;

        let size_str = String::from_utf8_lossy(&body[..line_end]);
        let size_str = size_str.split(';').next().unwrap_or("").trim();

        let size = usize::from_str_radix(size_str, 16)
            .map_err(|_| QueryError::InvalidResponse(format!("bad chunk size: {}", size_str)))?;

        body = &body[line_end + 2..];

        if size == 0 {
            return Ok(out);
        }

        if body.len() < size {
            return Err(QueryError::InvalidResponse("truncated chunk".to_string()));
        }

        out.extend_from_slice(&body[..size]);

        // Skip the chunk payload plus its trailing CRLF.
        body = &body[(size + 2).min(body.len())..];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_response() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"a\":1}";

        assert_eq!(parse_http_response(raw).unwrap(), "{\"a\":1}");
    }

    #[test]
    fn parses_chunked_response() {
        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nabcd\r\n3\r\nefg\r\n0\r\n\r\n";

        assert_eq!(parse_http_response(raw).unwrap(), "abcdefg");
    }

    #[test]
    fn rejects_error_status() {
        let raw = b"HTTP/1.1 404 Not Found\r\n\r\nnope";

        assert!(parse_http_response(raw).is_err());
    }
}
