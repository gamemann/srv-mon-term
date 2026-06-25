use std::net::{Ipv4Addr, ToSocketAddrs};

/// Resolves a hostname or IP address string to an `Ipv4Addr`.
///
/// # Arguments
/// * `host` - A string slice that holds the hostname or IP address to resolve.
///
/// # Returns
/// * `Option<Ipv4Addr>` - Returns `Some(Ipv4Addr)` if the resolution is successful, otherwise returns `None`.
pub fn resolve_to_ipv4(host: &str) -> Option<Ipv4Addr> {
    // Attempt to parse the host as an IPv4 address first
    if let Ok(ip) = host.parse::<Ipv4Addr>() {
        return Some(ip);
    }

    // If parsing fails, attempt to resolve the host to an IPv4 address using DNS
    format!("{}:0", host)
        .to_socket_addrs()
        .ok()?
        .find_map(|a| match a.ip() {
            std::net::IpAddr::V4(v4) => Some(v4),
            _ => None,
        })
}
