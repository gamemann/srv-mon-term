use anyhow::{Result, bail};

use crate::context::Context;
use crate::log_debug;
use crate::logger::level::LogLevel;
use crate::query::ext::QueryExt;
use crate::query::types::Query;
use crate::server::ServerCtx;
use crate::server::data::ServerStatus;
use crate::server::types::latency::ServerLatencyType;

impl ServerCtx {
    pub async fn query_server(&self, ctx: Context) -> Result<()> {
        let (tag, query_type, query_timeout) = {
            let server = self.server.read().await;

            let addr = format!("{}:{}", server.ip, server.port);

            let tag = format!("{} ({})", addr, server.query_type);
            let query_type = server.query_type.clone();
            let query_timeout = server.query_timeout;

            (tag, query_type, query_timeout)
        };

        // Create the query.
        let mut query = match Query::from_srv_type(&query_type).await {
            Ok(q) => {
                log_debug!(
                    ctx.logger.write().await,
                    "{}: Created query '{}'...",
                    tag,
                    query_type
                );

                q
            }
            Err(e) => bail!("Failed to create query: {}", e),
        };

        log_debug!(ctx.logger.write().await, "{}: Querying server info...", tag,);

        // Perform info query.
        let (latency_info, status, users_cur, users_max, map_name) = {
            let latency = {
                let mut server = self.server.write().await;

                let latency = match query.query_info(&mut server, query_timeout).await {
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

            // Set

            let (latency_type, status) = {
                let server = self.server.read().await;

                (server.latency_type.clone(), server.data.status.clone())
            };

            // If our server's query latency type is self info, we'll want to set the latency to the info query time.
            if latency_type == ServerLatencyType::SelfInfo {
                log_debug!(
                    ctx.logger.write().await,
                    "{}: Setting latency to info query latency of {}ms... Server status: {}",
                    tag,
                    latency,
                    status
                );

                self.add_latency(ctx.clone(), latency).await?;
            }

            let (users_cur, users_max, map_name) = {
                let server = self.server.read().await;

                let users_cur = server.data.users_cur;
                let users_max = server.data.users_max;
                let map_name = server.data.map_name.clone();

                (users_cur, users_max, map_name.clone())
            };

            (latency, status, users_cur, users_max, map_name)
        };

        // In basic mode, we print the info query with ping (trying to format it like a normal ping response).
        if ctx.args.basic {
            match status {
                ServerStatus::Online => {
                    println!(
                        "{}: Reply in {}ms. Users => {}/{}. Map => {}.",
                        tag,
                        latency_info,
                        users_cur,
                        users_max,
                        map_name.as_deref().unwrap_or("unknown")
                    );
                }
                ServerStatus::Offline => {
                    println!("{}: Timeout. Status: Offline.", tag);
                }
                ServerStatus::Error => {
                    println!("{}: Error during query. Status: Error.", tag);
                }
            }
        }

        log_debug!(
            ctx.logger.write().await,
            "{}: Querying server users...",
            tag
        );

        // Perform users query.
        {
            let (latency, latency_type) = {
                let mut server = self.server.write().await;

                let lat = match query.query_users(&mut server, query_timeout).await {
                    Ok(latency) => latency,
                    Err(e) => bail!("Failed to query server users: {}", e),
                };

                (lat, server.latency_type.clone())
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
        }

        log_debug!(ctx.logger.write().await, "{}: Querying server vars...", tag);

        // Perform vars query.
        {
            let (latency, latency_type) = {
                let mut server = self.server.write().await;

                let lat = match query.query_vars(&mut server, query_timeout).await {
                    Ok(latency) => latency,
                    Err(e) => bail!("Failed to query server vars: {}", e),
                };

                (lat, server.latency_type.clone())
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
        };

        Ok(())
    }
}
