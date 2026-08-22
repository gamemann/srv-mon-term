use anyhow::{Result, bail};

use crate::{
    cli::args::Args,
    context::Context,
    query::Query,
    server::types::{latency::ServerLatencyType, query::ServerQueryType},
    store::server::ServerStore,
    util::resolve_to_ipv4,
};

/// Parses `--dst` (and `--port` when the destination carries no port) into an address.
pub fn parse_dst(args: &Args) -> Result<Option<(String, u16)>> {
    let dst = match args.dst {
        Some(ref dst) => dst.trim(),
        None => return Ok(None),
    };

    if dst.is_empty() {
        return Ok(None);
    }

    let (host, port) = match dst.rsplit_once(':') {
        Some((host, port_str)) => {
            let port = port_str
                .parse::<u16>()
                .map_err(|_| anyhow::anyhow!("Malformed address: invalid port: {}", port_str))?;

            (host.to_string(), port)
        }
        None => {
            let port = match args.port {
                Some(port) => port,
                None => bail!("Missing port for address: {}", dst),
            };

            (dst.to_string(), port)
        }
    };

    if port == 0 {
        bail!("Malformed address: port cannot be 0");
    }

    // Make sure the host is usable, but keep what the user typed. Hostnames are resolved on
    // every query so servers behind round robin DNS or a proxy keep working.
    if resolve_to_ipv4(&host).is_none() {
        bail!("Malformed address: invalid IP or hostname: {}", host);
    }

    Ok(Some((host, port)))
}

/// Guesses the query type from the port when the user didn't pass `--query`.
///
/// Only called for servers we're adding, since we never want to silently change the protocol
/// of a server that is already in the store.
pub fn ensure_query_type(record: &mut ServerStore, args: &Args) -> Result<()> {
    if args.query.as_ref().is_some_and(|q| !q.trim().is_empty()) {
        return Ok(());
    }

    record.query_type = Query::get_query_type_from_port(record.port_query.unwrap_or(record.port))
        .ok_or_else(|| {
        anyhow::anyhow!(
            "Failed to determine query type from port {}. Please specify one via -q/--query.",
            record.port
        )
    })?;

    Ok(())
}

/// Applies every CLI override onto a server record.
pub fn apply_overrides(record: &mut ServerStore, args: &Args) -> Result<()> {
    if let Some(ref query) = args.query
        && !query.trim().is_empty()
    {
        record.query_type = query.parse::<ServerQueryType>().map_err(|_| {
            anyhow::anyhow!(
                "Unknown query type '{}'. Supported types: {}.",
                query,
                ServerQueryType::ALL
                    .iter()
                    .map(|t| t.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
    }

    if let Some(query_port) = args.query_port {
        record.port_query = Some(query_port);
    }

    if let Some(timeout) = args.timeout {
        record.query_timeout = timeout;
    }

    if let Some(interval) = args.query_interval {
        record.query_interval = interval;
    }

    if let Some(ref name) = args.name {
        record.display_name = Some(name.clone());
    }

    if let Some(ref latency_type) = args.latency_type {
        record.latency_type = latency_type
            .parse::<ServerLatencyType>()
            .map_err(|_| anyhow::anyhow!("Unknown latency type '{}'.", latency_type))?;
    }

    if let Some(interval) = args.latency_interval {
        record.latency_interval = Some(interval);
    }

    Ok(())
}

/// Builds a server record from the command line, if one was requested.
pub async fn check_server_cli(ctx: Context) -> Result<Option<ServerStore>> {
    let args = &ctx.args;

    let (ip, port) = match parse_dst(args)? {
        Some(addr) => addr,
        None => return Ok(None),
    };

    let mut record = ServerStore {
        ip,
        port,
        ..Default::default()
    };

    apply_overrides(&mut record, args)?;

    Ok(Some(record))
}
