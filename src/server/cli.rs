use anyhow::{Result, bail};

use crate::{
    context::Context,
    query::Query,
    server::{Server, types::query::ServerQueryType},
    util::resolve_to_ipv4,
};

pub async fn check_server_cli(ctx: Context) -> Result<Option<Server>> {
    let dst = match ctx.args.dst {
        Some(ref dst) => dst,
        None => return Ok(None),
    };

    // Retrieve IP and port.
    let (ip, port) = {
        // Check if the destination contains a colon, indicating an IP:port format.
        if dst.contains(':') {
            let parts: Vec<&str> = dst.split(':').collect();

            if parts.len() != 2 {
                bail!("Malformed address: {}", dst);
            }

            let ip = parts[0].to_string();
            let port_str = parts[1];

            let port = match port_str.parse::<u16>() {
                Ok(p) => p,
                Err(_) => bail!("Malformed address: invalid port: {}", port_str),
            };

            (ip, port)
        } else {
            // Otherwise, try to retrieve the port from the CLI arguments.
            let port = match ctx.args.port {
                Some(port) => port,
                None => bail!("Missing port for address: {}", dst),
            };

            (dst.to_string(), port)
        }
    };

    // Convert IP string to Ipv4Addr.
    let ip = match resolve_to_ipv4(&ip) {
        Some(ip) => ip,
        None => bail!("Malformed address: invalid IP or hostname: {}", ip),
    };

    let query_str = match ctx.args.query {
        Some(ref q) => q,
        None => &"".to_string(),
    };

    let query_type = match query_str.parse::<ServerQueryType>() {
        Ok(q) => Some(q),
        Err(_) => match Query::get_query_type_from_port(port) {
            Some(q) => Some(q),
            None => bail!(
                "Failed to determine query type from port {}. Please specify a specific query type via -q or --query.",
                port
            ),
        },
    };

    let server = {
        let mut server = Server::new(ip.to_string(), port, None);

        if let Some(ref q) = query_type {
            server.query_type = q.clone();
        }

        if let Some(timeout) = ctx.args.timeout {
            server.query_timeout = timeout;
        }

        server
    };

    Ok(Some(server))
}
