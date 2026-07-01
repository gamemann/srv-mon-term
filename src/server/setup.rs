use std::sync::Arc;

use anyhow::{Result, anyhow, bail};

use crate::context::Context;

use crate::log_trace;
use crate::server::check_server_cli;
use crate::server::context::ServerCtx;
use crate::store::ext::StoreExt;
use crate::store::server::ServerStore;
use crate::{log_info, logger::level::LogLevel};

pub async fn servers_setup_all(ctx: Context) -> Result<()> {
    let args = ctx.args.clone();

    // We need to modify the servers

    let mut servers_store = if !args.isolate {
        let mut store = ctx.store.write().await;

        match store.srv_fetch_all().await {
            Ok(res) => {
                log_info!(
                    ctx.logger.write().await,
                    "Fetched {} servers from store.",
                    res.len()
                );

                res
            }
            Err(e) => bail!("Failed to fetch servers from store: {}", e),
        }
    } else {
        Vec::new()
    };

    // Check for CLI override.
    let srv_cli = check_server_cli(ctx.clone()).await?;

    // First, check if we need to edit the CLI server in the store for the ID.
    if let Some(s) = &srv_cli {
        // Check if the server already exists in the store.
        let mut existing_server = servers_store
            .iter_mut()
            .find(|srv| srv.ip == s.ip && srv.port == s.port);

        // Check for deletion.
        let is_deleted = {
            let mut is_deleted = false;

            if args.delete {
                if let Some(existing) = existing_server {
                    let mut store = ctx.store.write().await;

                    store.srv_delete(existing).await.map_err(|e| {
                        anyhow!(
                            "Failed to delete server {}:{} from store: {}",
                            s.ip,
                            s.port,
                            e
                        )
                    })?;

                    log_info!(
                        ctx.logger.write().await,
                        "Deleted server {}:{} from store from CLI delete flag.",
                        s.ip,
                        s.port
                    );

                    // Server no longer exists.
                    existing_server = None;

                    is_deleted = true;
                } else {
                    bail!(
                        "Server {}:{} not found in the store for deletion.",
                        s.ip,
                        s.port
                    );
                }
            }

            is_deleted
        };

        if let Some(existing_server) = existing_server {
            // Update the existing server with CLI values.
            existing_server.query_type = s.query_type.clone();
            existing_server.query_timeout = s.query_timeout;
        } else if !is_deleted {
            // If it doesn't exist, add it to the list.
            servers_store.push(ServerStore {
                ip: s.ip.clone(),
                port: s.port,
                query_type: s.query_type.clone(),
                query_timeout: s.query_timeout,
                ..Default::default()
            });
        }
    }

    servers_store.dedup_by_key(|s| (s.ip.clone(), s.port));

    if servers_store.is_empty() {
        return Err(anyhow!(
            "No servers found in the store or provided via CLI."
        ));
    }

    log_trace!(ctx.logger.write().await, "Clearing servers...");

    // Reset the servers context vector.
    ctx.servers.write().await.clear();

    // We'll now want to loop through each server and spawn tasks required.
    for server in servers_store {
        let addr = format!("{}:{}", server.ip, server.port);

        log_trace!(
            ctx.logger.write().await,
            "Setting up server context for {}...",
            addr
        );

        let new_ctx = Arc::new(ServerCtx::new(
            Some(server.id.clone()),
            server.ip.clone(),
            server.port,
            server.port_query,
        ));

        match ServerCtx::add(new_ctx.clone(), ctx.clone()).await {
            Ok(_) => {
                let new_ctx = new_ctx.clone();

                log_trace!(
                    ctx.logger.write().await,
                    "Successfully set up server context for {}.",
                    addr
                );

                // Spawn tasks for the server.
                new_ctx
                    .setup_tasks(ctx.clone())
                    .await
                    .map_err(|e| anyhow!("Failed to setup tasks for server '{}': {}", addr, e))?;

                log_info!(
                    ctx.logger.write().await,
                    "Successfully set up server context and tasks for {}.",
                    addr
                );
            }
            Err(e) => {
                bail!("Failed to retrieve server context for '{}': {}", addr, e);
            }
        }

        if args.add
            && let Some(ref s) = srv_cli
        {
            if s.ip == server.ip && s.port == server.port {
                let s = s.clone();

                let mut store = ctx.store.write().await;

                store.srv_add(&server).await.map_err(|e| {
                    anyhow!("Failed to add server {}:{} to store: {}", s.ip, s.port, e)
                })?;

                log_info!(
                    ctx.logger.write().await,
                    "Added new server {}:{} to store from CLI add flag.",
                    s.ip,
                    s.port
                );
            }
        }

        // Add the server context to the main context's servers vector.
        ctx.servers.write().await.push(new_ctx);
    }

    Ok(())
}
