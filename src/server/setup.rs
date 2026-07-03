use std::sync::Arc;

use anyhow::{Result, anyhow, bail};

use crate::context::Context;

use crate::server::check_server_cli;
use crate::server::context::ServerCtx;
use crate::store::ext::StoreExt;
use crate::store::server::ServerStore;
use crate::{log_info, logger::level::LogLevel};
use crate::{log_trace, log_warn};

pub async fn servers_setup_all(ctx: Context) -> Result<()> {
    let args = ctx.args.clone();

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

        if !is_deleted {
            if let Some(existing_server) = existing_server {
                // Update the existing server with CLI values.
                existing_server.query_type = s.query_type.clone();
                existing_server.query_timeout = s.query_timeout;
            } else {
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

        let id = if server.id.trim().len() > 0 {
            Some(server.id.clone())
        } else {
            None
        };

        let new_ctx = Arc::new(ServerCtx::new(
            id,
            server.ip.clone(),
            server.port,
            server.port_query,
        ));

        log_trace!(
            ctx.logger.write().await,
            "Successfully created server context for {}",
            addr
        );

        match ServerCtx::add(new_ctx.clone(), ctx.clone()).await {
            Ok(_) => {
                log_trace!(
                    ctx.logger.write().await,
                    "Successfully added server context to main vector {}.",
                    addr
                );
            }
            Err(e) => {
                log_warn!(
                    ctx.logger.write().await,
                    "Failed to add server context for {}: {}",
                    addr,
                    e
                );
            }
        }

        let id_now = new_ctx.id.clone();

        // Spawn tasks for the server.
        new_ctx
            .setup_tasks(ctx.clone())
            .await
            .map_err(|e| anyhow!("Failed to setup tasks for server '{}': {}", addr, e))?;

        if args.add
            && let Some(ref s) = srv_cli
        {
            if s.ip == server.ip && s.port == server.port {
                let s = s.clone();

                let mut store = ctx.store.write().await;

                let updated_server = ServerStore {
                    id: id_now,
                    ..server
                };

                store.srv_add(&updated_server).await.map_err(|e| {
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
    }

    Ok(())
}
