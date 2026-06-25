use anyhow::{Result, anyhow, bail};

use crate::cli::QueryMonitor;
use crate::context::Context;
use crate::log_debug;
use crate::logger::level::LogLevel;
use crate::query::ext::QueryExt;
use crate::query::types::Query;
use crate::server::ServerCtx;
use crate::server::data::ServerStatus;
use crate::server::types::latency::ServerLatencyType;

pub struct QueryResponse {
    pub status: ServerStatus,

    pub latency_info: u64,
    pub latency_users: u64,
    pub latency_vars: u64,
}

impl ServerCtx {
    async fn print_query_info(&self, latency: u64, query_monitor: &QueryMonitor) {
        let (tag, data, users_len, vars_len) = {
            let server = self.server.read().await;

            let addr = format!("{}:{}", server.ip, server.port);

            let query_type = server.query_type.clone();

            let tag = format!("{} ({}:{})", addr, query_type, query_monitor.to_str());

            let users_len = server.data.users.len();
            let vars_len = server.data.vars.len();

            (tag, server.data.clone(), users_len, vars_len)
        };

        let mut content = String::new();

        content.push_str(&format!("{}: ", tag));

        let latency_ms = latency as f64 / 1000.0;

        match data.status {
            ServerStatus::Online => {
                content.push_str(&format!("Reply in {:.2}ms. ", latency_ms));

                match query_monitor {
                    QueryMonitor::Info => {
                        content.push_str(&format!(
                            "Game => {} (Dir => {}). ",
                            data.game_name.clone().unwrap_or("N/A".to_string()),
                            data.game_dir.clone().unwrap_or("N/A".to_string())
                        ));

                        content
                            .push_str(&format!("Users => {}/{}. ", data.users_cur, data.users_max));

                        if let Some(bots) = data.bots_cur {
                            content.push_str(&format!("Bots => {}. ", bots));
                        }

                        content.push_str(&format!(
                            "Map => {}. ",
                            data.map_name.unwrap_or("N/A".to_string())
                        ));

                        content.push_str(&format!(
                            "OS => {}. ",
                            data.os.map(|o| o.to_string()).unwrap_or("N/A".to_string())
                        ));

                        content.push_str(&format!(
                            "Secure => {}. ",
                            if data.is_secure { "Yes" } else { "No" }
                        ));

                        content.push_str(&format!(
                            "Dedicated => {}. ",
                            if data.is_dedicated { "Yes" } else { "No" }
                        ));

                        content.push_str(&format!(
                            "Public => {}. ",
                            if data.is_public { "Yes" } else { "No" }
                        ));

                        content.push_str(&format!(
                            "Version => {}. ",
                            data.version.unwrap_or("N/A".to_string())
                        ));
                    }
                    QueryMonitor::Users => {
                        content.push_str(&format!("Users => {}. ", users_len));
                    }
                    QueryMonitor::Vars => {
                        content.push_str(&format!("Vars => {}. ", vars_len));
                    }
                }
            }
            ServerStatus::Offline => {
                content.push_str("Offline");
            }
            ServerStatus::Error => {
                let err_code = data
                    .status_code
                    .map(|c| c.to_string())
                    .unwrap_or("N/A".to_string());

                content.push_str(&format!("Error => {}. ", err_code));
            }
        }

        println!("{}", content);
    }

    pub async fn query_server(&self, ctx: Context) -> Result<QueryResponse> {
        let (ip, port, addr, query_type, timeout) = {
            let server = self.server.read().await;

            let addr = format!("{}:{}", server.ip, server.port);

            let query_type = server.query_type.clone();
            let query_timeout = server.query_timeout;

            (
                server.ip.clone(),
                server.port,
                addr,
                query_type.clone(),
                query_timeout,
            )
        };

        let args = &ctx.args;
        let query_monitor = args
            .parse_query_monitor()
            .ok_or_else(|| anyhow!("Failed to parse query monitor"))?;

        // Format tag.
        let tag = format!("{} ({}:{})", addr, query_type, query_monitor.to_str());

        log_debug!(ctx.logger.write().await, "{}: Querying server info...", tag,);

        let monitor_only = args.use_query_monitor_only;

        // Perform info query.
        let func_info = async {
            let is_query_type = query_monitor == QueryMonitor::Info;

            if monitor_only && !is_query_type {
                log_debug!(
                    ctx.logger.write().await,
                    "{}: Skipping info query due to --monitor-only flag...",
                    tag
                );

                return Ok((0, ServerStatus::Offline, 0, 0, None));
            }

            // Create the query.
            let mut query = match Query::from_srv_type(&query_type).await {
                Ok(q) => {
                    log_debug!(
                        ctx.logger.write().await,
                        "{}: [INFO] Created query '{}'...",
                        tag,
                        query_type
                    );

                    q
                }
                Err(e) => bail!("Failed to create query: {}", e),
            };

            let latency = {
                let res = match query.query_info(&ip, port, timeout).await {
                    Ok(res) => res,
                    Err(e) => bail!("Failed to query info: {}", e),
                };

                // Start writing to server data.
                {
                    let mut server = self.server.write().await;

                    let data = &mut server.data;

                    let is_query_monitor = monitor_only && is_query_type;

                    if res.status != ServerStatus::Online && (!monitor_only || is_query_monitor) {
                        log_debug!(
                            ctx.logger.write().await,
                            "{}: Setting server status to {}...",
                            tag,
                            res.status
                        );

                        data.status_code = res.status_code.clone();
                        data.status = res.status.clone();
                    } else {
                        if data.status != ServerStatus::Online
                            && (!monitor_only || is_query_monitor)
                        {
                            log_debug!(
                                ctx.logger.write().await,
                                "{}: Setting server status to {}...",
                                tag,
                                res.status
                            );

                            data.status_code = res.status_code.clone();
                            data.status = res.status.clone();
                        }

                        data.srv_name = res.data.srv_name.clone();
                        data.map_name = res.data.map_name.clone();
                        data.game_name = res.data.game_name.clone();
                        data.game_dir = res.data.game_dir.clone();
                        data.game_id = res.data.game_id.clone();
                        data.users_cur = res.data.users_cnt;
                        data.users_max = res.data.users_max;
                        data.bots_cur = res.data.bots_cnt;
                        data.os = res.data.os.clone();
                        data.is_secure = res.data.is_secure;
                        data.is_dedicated = res.data.is_dedicated;
                        data.is_public = res.data.is_public;
                        data.version = res.data.version.clone();
                    }
                }

                res.latency
            };

            // Retrieve server info from info query.
            let (latency_type, status, users_cur, users_max, map_name) = {
                let server = self.server.read().await;

                let users_cur = server.data.users_cur;
                let users_max = server.data.users_max;
                let map_name = server.data.map_name.clone();

                (
                    server.latency_type.clone(),
                    server.data.status.clone(),
                    users_cur,
                    users_max,
                    map_name,
                )
            };

            // If our server's query latency type is self info, we'll want to set the latency to the info query time.
            if latency_type == ServerLatencyType::SelfInfo {
                log_debug!(
                    ctx.logger.write().await,
                    "{}: Setting latency to info query latency of {}ms... Server status: {}",
                    tag,
                    query_monitor.to_str(),
                    latency
                );

                self.add_latency(ctx.clone(), latency).await?;
            }

            Ok((latency, status, users_cur, users_max, map_name))
        };

        // Perform users query.
        let func_users = async {
            let is_query_type = query_monitor == QueryMonitor::Users;

            if monitor_only && !is_query_type {
                log_debug!(
                    ctx.logger.write().await,
                    "{}: Skipping users query due to --monitor-only flag...",
                    tag
                );

                return Ok((0, 0));
            }

            // Create the query.
            let mut query = match Query::from_srv_type(&query_type).await {
                Ok(q) => {
                    log_debug!(
                        ctx.logger.write().await,
                        "{}: [USERS] Created query '{}'...",
                        tag,
                        query_type
                    );

                    q
                }
                Err(e) => bail!("Failed to create query: {}", e),
            };

            let (latency, latency_type, users_count) = {
                let res = match query.query_users(&ip, port, timeout).await {
                    Ok(q) => q,
                    Err(e) => bail!("Failed to query server users: {}", e),
                };

                let check_offline = monitor_only && is_query_type;

                if res.data.users.len() > 0 || check_offline {
                    let mut server = self.server.write().await;

                    let data = &mut server.data;

                    if res.status != ServerStatus::Online && check_offline {
                        log_debug!(
                            ctx.logger.write().await,
                            "{}: Setting server status to {}...",
                            tag,
                            res.status
                        );

                        data.status_code = res.status_code.clone();
                        data.status = res.status.clone();
                    } else {
                        data.users = res.data.users.clone();
                    }
                }

                let latency_type = {
                    let server = self.server.read().await;

                    server.latency_type.clone()
                };

                (res.latency, latency_type, res.data.users.len())
            };

            log_debug!(
                ctx.logger.write().await,
                "{}: Queried server users in {}ms...",
                tag,
                latency
            );

            if latency_type == ServerLatencyType::SelfUsers {
                log_debug!(
                    ctx.logger.write().await,
                    "{}: Setting latency to users query latency of {}ms...",
                    tag,
                    latency
                );

                self.add_latency(ctx.clone(), latency).await?;
            }

            Ok((latency, users_count))
        };

        // Perform vars query.
        let func_vars = async {
            let is_query_type = query_monitor == QueryMonitor::Vars;

            if monitor_only && !is_query_type {
                log_debug!(
                    ctx.logger.write().await,
                    "{}: Skipping vars query due to --monitor-only flag...",
                    tag
                );

                return Ok((0, 0));
            }

            // Create the query.
            let mut query = match Query::from_srv_type(&query_type).await {
                Ok(q) => {
                    log_debug!(
                        ctx.logger.write().await,
                        "{}: [VARS] Created query '{}'...",
                        tag,
                        query_type
                    );

                    q
                }
                Err(e) => bail!("Failed to create query: {}", e),
            };

            let (latency, latency_type, vars_count) = {
                let res = match query.query_vars(&ip, port, timeout).await {
                    Ok(q) => q,
                    Err(e) => bail!("Failed to query server vars: {}", e),
                };

                let check_offline = monitor_only && is_query_type;

                if res.data.vars.len() > 0 || check_offline {
                    let mut server = self.server.write().await;

                    let data = &mut server.data;

                    if res.status != ServerStatus::Online && check_offline {
                        log_debug!(
                            ctx.logger.write().await,
                            "{}: Setting server status to {}...",
                            tag,
                            res.status
                        );

                        data.status_code = res.status_code.clone();
                        data.status = res.status.clone();
                    } else {
                        data.vars = res.data.vars.clone();
                    }
                }

                let latency_type = {
                    let server = self.server.read().await;

                    server.latency_type.clone()
                };

                (res.latency, latency_type, res.data.vars.len() as u32)
            };

            log_debug!(
                ctx.logger.write().await,
                "{}: Queried server vars in {}ms...",
                tag,
                latency
            );

            if latency_type == ServerLatencyType::SelfVars {
                log_debug!(
                    ctx.logger.write().await,
                    "{}: Setting latency to vars query latency of {}ms...",
                    tag,
                    latency
                );

                self.add_latency(ctx.clone(), latency).await?;
            }

            Ok((latency, vars_count))
        };

        let (info, users, vars) = tokio::try_join!(func_info, func_users, func_vars)?;

        let (latency_info, status, _, _, _) = info;
        let (latency_users, _) = users;
        let (latency_vars, _) = vars;

        if args.basic {
            let latency_to_use = match query_monitor {
                QueryMonitor::Info => latency_info,
                QueryMonitor::Users => latency_users,
                QueryMonitor::Vars => latency_vars,
            };

            self.print_query_info(latency_to_use, &query_monitor).await;
        }

        Ok(QueryResponse {
            status,
            latency_info,
            latency_users,
            latency_vars,
        })
    }
}
