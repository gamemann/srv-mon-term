use std::{net::Ipv4Addr, sync::Arc};

use anyhow::{Result, anyhow, bail};

use crate::{
    context::Context,
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

    let add = ctx.args.add;
    let delete = ctx.args.delete;

    // Attempt to find the server context for the provided IP and port.
    let srv_ctx = match ServerCtx::get_server_ctx_by_addr(ctx.clone(), &ip.to_string(), port).await
    {
        Ok(sctx) => sctx.clone(),
        Err(_) => {
            // Error just indicates that the server context doesn't exist, so we'll want to add it if we're adding.
            if add {
                // If the server context doesn't exist and we're adding, create it.
                let new_srv_ctx = ServerCtx::new(ip.to_string(), port, None);

                if let Some(ref q) = query_type {
                    new_srv_ctx.server.write().await.query_type = q.clone();
                }

                let new_srv_ctx = Arc::new(new_srv_ctx);

                ctx.servers.write().await.push(new_srv_ctx.clone());

                // We need to spawn the tasks and such.
                {
                    let new_srv_ctx = new_srv_ctx.clone();

                    new_srv_ctx.add(ctx.clone()).await?;
                }

                new_srv_ctx
            } else if delete {
                bail!("Failed to find server with IP {} and port {}", ip, port)
            } else {
                return Ok(());
            }
        }
    };

    if let Some(ref q) = query_type {
        let mut server = srv_ctx.server.write().await;

        server.query_type = q.clone();
    }

    Ok(())
}
