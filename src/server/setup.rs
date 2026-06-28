use std::sync::Arc;

use anyhow::{Result, anyhow, bail};
use tokio::sync::RwLock;

use crate::context::Context;

use crate::server::context::ServerCtx;
use crate::store::ext::StoreExt;

pub async fn servers_setup_all(ctx: Context) -> Result<()> {
    // If we're in isolate mode, we don't want to setup all servers.
    if ctx.args.isolate {
        return Ok(());
    }

    let servers = {
        let mut store = ctx.store.write().await;

        // Reset the context servers.
        ctx.servers.write().await.clear();

        // Retrieve all servers from the store.
        match store.srv_fetch_all().await {
            Ok(res) => res,
            Err(e) => bail!("Failed to fetch servers: {}", e),
        }
    };

    // We'll now want to loop through each server and spawn tasks required.
    for server in servers {
        let addr = format!("{}:{}", server.ip, server.port);

        // Push server to context.
        ctx.servers.write().await.push(Arc::new(ServerCtx {
            id: server.id.clone(),
            server: RwLock::new(server.clone()),
            tasks: Default::default(),
            latency: Default::default(),
        }));

        // Retrieve the server context we just created.
        let server_ctx = match ctx.get_server_ctx(&server).await {
            Some(ctx) => ctx,
            None => bail!("Failed to find server context for server '{}'", addr),
        };

        // Spawn tasks for the server.
        server_ctx
            .setup_tasks(ctx.clone())
            .await
            .map_err(|e| anyhow!("Failed to setup tasks for server '{}': {}", addr, e))?;
    }

    Ok(())
}
