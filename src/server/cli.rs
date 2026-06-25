use std::{net::Ipv4Addr, sync::Arc};

use anyhow::{Result, anyhow, bail};

use crate::{
    context::Context,
    log_info,
    logger::level::LogLevel,
    server::{ServerCtx, types::query::ServerQueryType},
};

pub async fn check_server_cli(ctx: Context) -> Result<()> {
    let ip = match ctx.args.dst {
        Some(ref dst) => dst.parse::<Ipv4Addr>(),
        None => return Ok(()),
    }
    .map_err(|e| anyhow!("Failed to parse destination IP address: {}", e))?;

    let port = match ctx.args.port {
        Some(port) => port,
        None => return Ok(()),
    };

    let query_str = match ctx.args.query {
        Some(ref q) => q,
        None => return Ok(()),
    };

    let query_type = match query_str.parse::<ServerQueryType>() {
        Ok(q) => Some(q),
        Err(_) => bail!("Invalid query type: {}", query_str),
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
