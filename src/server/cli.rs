use std::sync::Arc;

use anyhow::{Result, bail};

use crate::{
    context::Context,
    log_info,
    logger::level::LogLevel,
    query::Query,
    server::{ServerCtx, types::query::ServerQueryType},
    util::resolve_to_ipv4,
};

pub async fn check_server_cli(ctx: Context) -> Result<()> {
    let dst = match ctx.args.dst {
        Some(ref dst) => dst,
        None => return Ok(()),
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

    log_info!(
        ctx.logger.write().await,
        "Found server through CLI: {}:{}. Query type: {:?}. Attempting to find server context...",
        ip,
        port,
        query_type
    );

    let add = ctx.args.add;
    let delete = ctx.args.delete;

    // Attempt to find the server context for the provided IP and port.
    let srv_ctx = match ServerCtx::get_server_ctx_by_addr(ctx.clone(), &ip.to_string(), port).await
    {
        Ok(sctx) => {
            // If we're in isolation mode, we need to setup the tasks.
            if ctx.args.isolate {
                sctx.clone().setup_tasks(ctx.clone()).await?;
            }

            sctx.clone()
        }
        Err(_) => {
            if delete {
                bail!(
                    "Failed to find server context for {}:{}. Cannot delete server that doesn't exist.",
                    ip,
                    port
                );
            }

            // If the server context doesn't exist and we're adding, create it.
            let new_srv_ctx = ServerCtx::new(ip.to_string(), port, None);

            if let Some(ref q) = query_type {
                new_srv_ctx.server.write().await.query_type = q.clone();
            }

            let new_srv_ctx = Arc::new(new_srv_ctx);
            {
                let mut servers = ctx.servers.write().await;

                servers.push(new_srv_ctx.clone());
            }

            // Add the server to the store if the flag is set.
            if add {
                let new_srv_ctx = new_srv_ctx.clone();

                new_srv_ctx.add(ctx.clone()).await?;
            }

            // Setup tasks.
            new_srv_ctx.clone().setup_tasks(ctx.clone()).await?;

            new_srv_ctx
        }
    };

    {
        let mut server = srv_ctx.server.write().await;

        if let Some(ref q) = query_type {
            server.query_type = q.clone();
        }

        if let Some(timeout) = ctx.args.timeout {
            server.query_timeout = timeout;
        }
    }

    Ok(())
}
