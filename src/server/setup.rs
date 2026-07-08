use std::sync::Arc;

use anyhow::{Result, anyhow, bail};

use crate::cli::QueryMonitor;
use crate::context::Context;

use crate::server::check_server_cli;
use crate::server::context::ServerCtx;
use crate::server::types::latency::ServerLatencyType;
use crate::store::ext::StoreExt;
use crate::store::server::ServerStore;
use crate::{log_info, logger::level::LogLevel, logger::Logger};
use crate::{log_trace, log_warn};

pub async fn servers_setup_all(ctx: Context) -> Result<()> {
    let args = ctx.args.clone();

    let mut servers_store = if !args.isolate {
        let store = ctx.store.read().await;

        match store.srv_fetch_all().await {
            Ok(res) => {
                log_info!(ctx,
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

                    log_info!(ctx,
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

    log_trace!(ctx, "Clearing servers...");

    // Reset the servers context vector.
    ctx.servers.write().await.clear();

    // We'll now want to loop through each server and spawn tasks required.
    for server in servers_store {
        let addr = format!("{}:{}", server.ip, server.port);

        log_trace!(ctx,
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

        log_trace!(ctx,
            "Successfully created server context for {}",
            addr
        );

        match ServerCtx::add(new_ctx.clone(), ctx.clone()).await {
            Ok(_) => {
                log_trace!(ctx,
                    "Successfully added server context to main vector {}.",
                    addr
                );
            }
            Err(e) => {
                log_warn!(ctx,
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

        // Make sure we save the server to the store if the CLI flag is set and the server matches the CLI server.
        if args.save
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

                log_info!(ctx,
                    "Added new server {}:{} to store from CLI add flag.",
                    s.ip,
                    s.port
                );
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
                // We need to loop through all servers and log their latency summaries.
                let servers = ctx.servers.read().await;

                log_info!(ctx,
                    "Scheduler shutdown initiated. Logging latency summaries for all servers..."
                );

                for srv_ctx in servers.iter() {
                    let self_clone = srv_ctx.clone();
                    let ctx = ctx.clone();

                    let addr = {
                        let server = self_clone.server.read().await;

                        format!("{}:{}", server.ip, server.port)
                    };

                    let query_monitor = ctx.args.parse_query_monitor().unwrap_or(QueryMonitor::Info);
                    
                    // Retrieve minimum, maximum and average latency for each type and log it.
                    let (info_min, info_max, info_avg) = if query_monitor == QueryMonitor::Info {
                        
                        let history = self_clone.latency.read().await;

                        let history = history
                            .iter()
                            .filter(|x| (x.type_ == ServerLatencyType::A2sInfo || x.type_ == ServerLatencyType::SelfInfo) && x.val > 0)
                            .collect::<Vec<_>>();

                        let min = history
                            .iter()
                            .min_by_key(|l| l.val)
                            .map(|l| l.val)
                            .unwrap_or(0);

                        let max = history
                            .iter()
                            .max_by_key(|l| l.val)
                            .map(|l| l.val)
                            .unwrap_or(0);

                        let avg = if history.len() > 0 {
                            history.iter().map(|l| l.val).sum::<u64>() / history.len() as u64
                        } else {
                            0
                        };

                        (
                            min as f64 / 1000.0,
                            max as f64 / 1000.0,
                            avg as f64 / 1000.0,
                        )
                    } else { 
                        (0.0, 0.0, 0.0)
                    };

                    let (users_min, users_max, users_avg) = if query_monitor == QueryMonitor::Users {
                        let history = self_clone.latency.read().await;

                        let history = history
                            .iter()
                            .filter(|x| (x.type_ == ServerLatencyType::A2sPlayers || x.type_ == ServerLatencyType::SelfUsers) && x.val > 0)
                            .collect::<Vec<_>>();

                        let min = history
                            .iter()
                            .min_by_key(|l| l.val)
                            .map(|l| l.val)
                            .unwrap_or(0);

                        let max = history
                            .iter()
                            .max_by_key(|l| l.val)
                            .map(|l| l.val)
                            .unwrap_or(0);

                        let avg = if history.len() > 0 {
                            history.iter().map(|l| l.val).sum::<u64>() / history.len() as u64
                        } else {
                            0
                        };

                        (
                            min as f64 / 1000.0,
                            max as f64 / 1000.0,
                            avg as f64 / 1000.0,
                        )
                    } else {
                        (0.0, 0.0, 0.0)
                    };

                    let (vars_min, vars_max, vars_avg) = if query_monitor == QueryMonitor::Vars {
                        let history = self_clone.latency.read().await;

                        let history = history
                            .iter()
                            .filter(|x| (x.type_ == ServerLatencyType::A2sRules || x.type_ == ServerLatencyType::SelfVars) && x.val > 0)
                            .collect::<Vec<_>>();

                        let min = history
                            .iter()
                            .min_by_key(|l| l.val)
                            .map(|l| l.val)
                            .unwrap_or(0);

                        let max = history
                            .iter()
                            .max_by_key(|l| l.val)
                            .map(|l| l.val)
                            .unwrap_or(0);

                        let avg = if history.len() > 0 {
                            history.iter().map(|l| l.val).sum::<u64>() / history.len() as u64
                        } else {
                            0
                        };

                        (
                            min as f64 / 1000.0,
                            max as f64 / 1000.0,
                            avg as f64 / 1000.0,
                        )
                    } else {
                        (0.0, 0.0, 0.0)
                    };

                    match query_monitor {
                        QueryMonitor::Info => {
                            if info_avg > 0.0 {
                                log_info!(ctx,
                                    "Latency summary for server {} (Info): min:{:.2}ms, max: {:.2}ms, avg: {:.2}ms",
                                    addr,
                                    info_min,
                                    info_max,
                                    info_avg
                                );
                            } else {
                                log_info!(ctx,
                                    "Latency summary for server {} (Info): No data available (Offline?)",
                                    addr
                                );
                            }
                        }
                        QueryMonitor::Users => {
                            if users_avg > 0.0 {
                                log_info!(ctx,
                                    "Latency summary for server {} (Users): min:{:.2}ms, max: {:.2}ms, avg: {:.2}ms",
                                    addr,
                                    users_min,
                                    users_max,
                                    users_avg
                                );
                            } else {
                                log_info!(ctx,
                                    "Latency summary for server {} (Users): No data available (Offline?)",
                                    addr
                                );
                            }
                        }
                        QueryMonitor::Vars => {
                            if vars_avg > 0.0 {
                                log_info!(ctx,
                                    "Latency summary for server {} (Vars): min:{:.2}ms, max: {:.2}ms, avg: {:.2}ms",
                                    addr,
                                    vars_min,
                                    vars_max,
                                    vars_avg
                                );
                            } else {
                                log_info!(ctx,
                                    "Latency summary for server {} (Vars): No data available (Offline?)",
                                    addr
                                );
                            }
                        }
                    }
                }
            })
        }));
    }

    Ok(())
}
