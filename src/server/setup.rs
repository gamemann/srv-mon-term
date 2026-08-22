use std::collections::HashSet;
use std::sync::Arc;

use anyhow::{Result, anyhow, bail};
use uuid::Uuid;

use crate::cli::QueryMonitor;
use crate::context::Context;

use crate::server::cli::{apply_overrides, check_server_cli, ensure_query_type};
use crate::server::context::ServerCtx;
use crate::server::types::Server;
use crate::server::types::latency::ServerLatencyType;
use crate::store::ext::StoreExt;
use crate::store::server::ServerStore;
use crate::{log_info, log_trace, log_warn, logger::Logger, logger::level::LogLevel};

/// Latency sources that belong to each monitored query.
fn latency_types_for(monitor: &QueryMonitor) -> [ServerLatencyType; 2] {
    match monitor {
        QueryMonitor::Info => [ServerLatencyType::SelfInfo, ServerLatencyType::A2sInfo],
        QueryMonitor::Users => [ServerLatencyType::SelfUsers, ServerLatencyType::A2sPlayers],
        QueryMonitor::Vars => [ServerLatencyType::SelfVars, ServerLatencyType::A2sRules],
    }
}

/// Merges the server described on the command line into the list loaded from the store.
async fn merge_cli_server(ctx: &Context, servers: &mut Vec<ServerStore>) -> Result<()> {
    let args = &ctx.args;

    let mut cli_server = match check_server_cli(ctx.clone()).await? {
        Some(server) => server,
        None => return Ok(()),
    };

    let existing = servers
        .iter()
        .position(|srv| srv.ip == cli_server.ip && srv.port == cli_server.port);

    // Deletion only needs the address.
    if args.delete {
        let idx = match existing {
            Some(idx) => idx,
            None => bail!(
                "Server {} not found in the store for deletion.",
                cli_server.to_addr()
            ),
        };

        let record = servers.remove(idx);

        {
            let mut store = ctx.store.write().await;

            store.srv_delete(&record).await.map_err(|e| {
                anyhow!(
                    "Failed to delete server {} from store: {}",
                    record.to_addr(),
                    e
                )
            })?;
        }

        log_info!(ctx, "Deleted server {} from store.", record.to_addr());

        return Ok(());
    }

    match existing {
        // Apply the overrides on top of what we already know about the server.
        Some(idx) => {
            let record = &mut servers[idx];

            let id = record.id.clone();

            apply_overrides(record, args)?;

            record.id = id;

            if args.save {
                let record = record.clone();

                let mut store = ctx.store.write().await;

                store.srv_update(&record).await.map_err(|e| {
                    anyhow!(
                        "Failed to update server {} in store: {}",
                        record.to_addr(),
                        e
                    )
                })?;

                log_info!(ctx, "Updated server {} in store.", record.to_addr());
            }
        }
        None => {
            ensure_query_type(&mut cli_server, args)?;

            cli_server.id = Uuid::now_v7().to_string();

            if args.save {
                let mut store = ctx.store.write().await;

                store.srv_add(&cli_server).await.map_err(|e| {
                    anyhow!(
                        "Failed to add server {} to store: {}",
                        cli_server.to_addr(),
                        e
                    )
                })?;

                log_info!(ctx, "Added server {} to store.", cli_server.to_addr());
            }

            servers.push(cli_server);
        }
    }

    Ok(())
}

/// Sets up every server we should monitor and returns how many were started.
/// Registers a server while the program is running: persists it (when asked), builds its
/// context and starts its query/latency tasks.
pub async fn server_add(
    ctx: Context,
    mut record: ServerStore,
    persist: bool,
) -> Result<Arc<ServerCtx>> {
    record.ip = record.ip.trim().to_string();

    if record.ip.is_empty() {
        bail!("A destination address is required.");
    }

    if record.port == 0 {
        bail!("A valid port is required.");
    }

    // Reject addresses we already monitor so we don't end up querying twice.
    {
        let servers = ctx.servers.read().await;

        for srv_ctx in servers.iter() {
            let server = srv_ctx.server.read().await;

            if server.ip == record.ip && server.port == record.port {
                bail!("{} is already being monitored.", record.to_addr());
            }
        }
    }

    if record.id.trim().is_empty() {
        record.id = Uuid::now_v7().to_string();
    }

    if persist {
        let mut store = ctx.store.write().await;

        store
            .srv_add(&record)
            .await
            .map_err(|e| anyhow!("Failed to add server {} to store: {}", record.to_addr(), e))?;
    }

    let addr = record.to_addr();
    let id = record.id.clone();

    let srv_ctx = Arc::new(ServerCtx::from_server(Some(id), Server::from(record)));

    ServerCtx::add(srv_ctx.clone(), ctx.clone())
        .await
        .map_err(|e| anyhow!("Failed to add server context for {}: {}", addr, e))?;

    srv_ctx
        .clone()
        .setup_tasks(ctx.clone())
        .await
        .map_err(|e| anyhow!("Failed to setup tasks for server '{}': {}", addr, e))?;

    log_info!(ctx, "Now monitoring {}.", addr);

    Ok(srv_ctx)
}

pub async fn servers_setup_all(ctx: Context) -> Result<usize> {
    let args = ctx.args.clone();

    let mut servers = if !args.isolate {
        let store = ctx.store.read().await;

        match store.srv_fetch_all().await {
            Ok(res) => {
                log_info!(ctx, "Fetched {} servers from store.", res.len());

                res
            }
            Err(e) => bail!("Failed to fetch servers from store: {}", e),
        }
    } else {
        Vec::new()
    };

    merge_cli_server(&ctx, &mut servers).await?;

    // Drop duplicate addresses regardless of the order they came back in.
    let mut seen = HashSet::new();
    servers.retain(|srv| seen.insert((srv.ip.clone(), srv.port)));

    if args.isolate
        && let Some(ref dst) = args.dst
    {
        log_trace!(ctx, "Isolating to server from CLI ({}).", dst);
    }

    // Having nothing to monitor isn't an error: the TUI can add servers at runtime.
    if servers.is_empty() {
        log_info!(ctx, "No servers to monitor yet.");

        return Ok(0);
    }

    log_trace!(ctx, "Clearing servers...");

    // Reset the servers context vector.
    ctx.servers.write().await.clear();

    let mut total = 0;

    for record in servers {
        let addr = record.to_addr();

        log_trace!(ctx, "Setting up server context for {}...", addr);

        // Already persisted (or intentionally CLI only), so never write it back here.
        match server_add(ctx.clone(), record, false).await {
            Ok(_) => total += 1,
            Err(e) => {
                log_warn!(ctx, "Failed to set up server {}: {}", addr, e);
            }
        }
    }

    // Setup scheduler shutdown handler here.
    {
        // We need to clone a separate context due to sch borrowing it.
        let shutdown_ctx = ctx.clone();

        let mut sch = ctx.sch.write().await;

        sch.set_shutdown_handler(Box::new(move || {
            let ctx = shutdown_ctx.clone();

            Box::pin(async move {
                let servers = ctx.servers.read().await;

                log_info!(
                    ctx,
                    "Scheduler shutdown initiated. Logging latency summaries for all servers..."
                );

                let query_monitor = ctx.args.parse_query_monitor().unwrap_or(QueryMonitor::Info);
                let types = latency_types_for(&query_monitor);

                for srv_ctx in servers.iter() {
                    let addr = srv_ctx.server.read().await.to_addr();

                    match srv_ctx.latency_summary(&types).await {
                        Some(summary) => {
                            log_info!(
                                ctx,
                                "Latency summary for server {} ({}): min: {:.2}ms, max: {:.2}ms, avg: {:.2}ms over {} samples",
                                addr,
                                query_monitor.to_str(),
                                summary.min,
                                summary.max,
                                summary.avg,
                                summary.samples
                            );
                        }
                        None => {
                            log_info!(
                                ctx,
                                "Latency summary for server {} ({}): No data available (Offline?)",
                                addr,
                                query_monitor.to_str()
                            );
                        }
                    }
                }
            })
        }));
    }

    Ok(total)
}
