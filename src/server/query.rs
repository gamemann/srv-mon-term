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
    fn print_query_info(
        tag: &str,
        status: &ServerStatus,
        latency: u64,

        users_cur: Option<u16>,
        users_max: Option<u16>,
        map_name: Option<String>,

        vars_max: Option<u32>,
    ) {
        let mut content = String::new();

        content.push_str(&format!("{}: ", tag));

        let latency_ms = latency as f64 / 1000.0;

        match status {
            ServerStatus::Online => {
                content.push_str(&format!("Reply in {:.2}ms. ", latency_ms));

                if let Some(users_cur) = users_cur {
                    if let Some(users_max) = users_max {
                        content.push_str(&format!("Users => {}/{}. ", users_cur, users_max));
                    } else {
                        content.push_str(&format!("Users => {}. ", users_cur));
                    }
                }

                if let Some(map_name) = map_name {
                    content.push_str(&format!("Map => {}. ", map_name));
                }

                if let Some(vars_max) = vars_max {
                    content.push_str(&format!("Vars => {}.", vars_max));
                }
            }
            ServerStatus::Offline => {
                content.push_str("Offline");
            }
            ServerStatus::Error => {
                content.push_str("Error");
            }
        }

        println!("{}", content);
    }

    pub async fn query_server(&self, ctx: Context) -> Result<QueryResponse> {
        let (addr, query_type, query_timeout) = {
            let server = self.server.read().await;

            let addr = format!("{}:{}", server.ip, server.port);

            let query_type = server.query_type.clone();
            let query_timeout = server.query_timeout;

            (addr, query_type, query_timeout)
        };

        let args = &ctx.args;
        let query_monitor = args
            .parse_query_monitor()
            .ok_or_else(|| anyhow!("Failed to parse query monitor"))?;

        // Format tag.
        let tag = format!("{} ({}:{})", addr, query_type, query_monitor.to_str());

        log_debug!(ctx.logger.write().await, "{}: Querying server info...", tag,);

        // Perform info query.
        let func_info = async {
            if args.use_query_monitor_only && query_monitor != QueryMonitor::Info {
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
                let mut server = self.server.write().await;

                let latency = match query
                    .query_info(
                        &mut server,
                        query_timeout,
                        query_monitor == QueryMonitor::Info,
                    )
                    .await
                {
                    Ok(latency) => latency,
                    Err(e) => bail!("Failed to query info: {}", e),
                };

                // Set status to online if we got a successful response.
                if server.data.status != ServerStatus::Online {
                    log_debug!(
                        ctx.logger.write().await,
                        "{}: Setting server status to Online...",
                        tag
                    );

                    server.data.status = ServerStatus::Online;
                }

                latency
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
            if args.use_query_monitor_only && query_monitor != QueryMonitor::Users {
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
                let mut server = self.server.write().await;

                let lat = match query
                    .query_users(
                        &mut server,
                        query_timeout,
                        query_monitor == QueryMonitor::Users,
                    )
                    .await
                {
                    Ok(latency) => latency,
                    Err(e) => bail!("Failed to query server users: {}", e),
                };

                (lat, server.latency_type.clone(), server.data.users.len())
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
            if args.use_query_monitor_only && query_monitor != QueryMonitor::Vars {
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
                let mut server = self.server.write().await;

                let lat = match query
                    .query_vars(
                        &mut server,
                        query_timeout,
                        query_monitor == QueryMonitor::Vars,
                    )
                    .await
                {
                    Ok(latency) => latency,
                    Err(e) => bail!("Failed to query server vars: {}", e),
                };

                (
                    lat,
                    server.latency_type.clone(),
                    server.data.vars.len() as u32,
                )
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

        let (latency_info, status, users_cur, users_max, map_name) = info;
        let (latency_users, users_count) = users;
        let (latency_vars, vars_count) = vars;

        if args.basic {
            let users_cnt = match query_monitor {
                QueryMonitor::Info => Some(users_cur),
                QueryMonitor::Users => Some(users_count as u16),
                QueryMonitor::Vars => None,
            };

            let users_max = if query_monitor == QueryMonitor::Info {
                Some(users_max)
            } else {
                None
            };

            let map_name = if query_monitor == QueryMonitor::Info {
                map_name
            } else {
                None
            };

            let vars_count = if query_monitor == QueryMonitor::Vars {
                Some(vars_count)
            } else {
                None
            };

            Self::print_query_info(
                &tag,
                &status,
                latency_info,
                users_cnt,
                users_max,
                map_name,
                vars_count,
            );
        }

        Ok(QueryResponse {
            status,
            latency_info,
            latency_users,
            latency_vars,
        })
    }
}
