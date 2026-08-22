use anyhow::{Result, anyhow};

use crate::cli::QueryMonitor;
use crate::context::Context;
use crate::log_debug;
use crate::logger::Logger;
use crate::logger::level::LogLevel;
use crate::query::ext::QueryExt;
use crate::query::types::Query;
use crate::query::types::ext::{
    InfoResponse, QueryResponse as ProtoResponse, UsersResponse, VarsResponse,
};
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
    async fn print_query_info(&self, latency: f64, query_monitor: &QueryMonitor) {
        let (tag, data, users_len, vars_len) = {
            let server = self.server.read().await;

            let query_type = server.query_type;

            let tag = format!(
                "{} ({}:{})",
                server.to_addr(),
                query_type,
                query_monitor.to_str()
            );

            let users_len = server.data.users.len();
            let vars_len = server.data.vars.len();

            (tag, server.data.clone(), users_len, vars_len)
        };

        let status = {
            let statuses = self.statuses.read().await;

            match query_monitor {
                QueryMonitor::Info => statuses.query_info.clone(),
                QueryMonitor::Users => statuses.query_users.clone(),
                QueryMonitor::Vars => statuses.query_vars.clone(),
            }
        };

        let mut content = String::new();

        content.push_str(&format!("{}: ", tag));

        match status {
            ServerStatus::Online => {
                content.push_str(&format!("Reply in {:.2}ms. ", latency));

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
            ServerStatus::Error(code) => {
                content.push_str(&format!("Error => {}. ", code));
            }
            ServerStatus::Unknown => {
                content.push_str("Unknown");
            }
        }

        println!("{}", content);
    }

    /// Stores the result of an info query and records its latency when configured to do so.
    async fn apply_info(
        &self,
        ctx: &Context,
        res: &ProtoResponse<InfoResponse>,
        queried_port: u16,
    ) -> Result<()> {
        {
            let mut statuses = self.statuses.write().await;

            statuses.query_info = res.status.clone();
        }

        if res.status == ServerStatus::Online {
            let mut server = self.server.write().await;

            let data = &mut server.data;

            data.srv_name = res.data.srv_name.clone();
            data.map_name = res.data.map_name.clone();
            data.game_name = res.data.game_name.clone();
            data.game_dir = res.data.game_dir.clone();
            data.game_id = res.data.game_id;
            data.users_cur = res.data.users_cnt;
            data.users_max = res.data.users_max;
            data.bots_cur = res.data.bots_cnt;
            data.os = res.data.os.clone();
            data.is_secure = res.data.is_secure;
            data.is_dedicated = res.data.is_dedicated;
            data.is_public = res.data.is_public;
            data.version = res.data.version.clone();
            data.last_updated = Some(chrono::Utc::now().timestamp_millis() as u64);

            // Some protocols report the port players connect to, which can differ from the
            // port that answers queries. Keep querying the port that just answered us.
            if let Some(game_port) = res.data.game_port
                && game_port != 0
                && game_port != queried_port
            {
                server.port = game_port;
                server.port_query = Some(queried_port);
            }
        }

        self.record_latency(
            ctx,
            ServerLatencyType::SelfInfo,
            res.status.clone(),
            res.latency,
        )
        .await
    }

    /// Stores the result of a users query and records its latency when configured to do so.
    async fn apply_users(&self, ctx: &Context, res: &ProtoResponse<UsersResponse>) -> Result<()> {
        {
            let mut statuses = self.statuses.write().await;

            statuses.query_users = res.status.clone();
        }

        if res.status == ServerStatus::Online {
            let mut server = self.server.write().await;

            server.data.users = res.data.users.clone();
        }

        self.record_latency(
            ctx,
            ServerLatencyType::SelfUsers,
            res.status.clone(),
            res.latency,
        )
        .await
    }

    /// Stores the result of a vars query and records its latency when configured to do so.
    async fn apply_vars(&self, ctx: &Context, res: &ProtoResponse<VarsResponse>) -> Result<()> {
        {
            let mut statuses = self.statuses.write().await;

            statuses.query_vars = res.status.clone();
        }

        if res.status == ServerStatus::Online {
            let mut server = self.server.write().await;

            server.data.vars = res.data.vars.clone();
        }

        self.record_latency(
            ctx,
            ServerLatencyType::SelfVars,
            res.status.clone(),
            res.latency,
        )
        .await
    }

    /// Adds a latency sample when the server tracks latency through this specific query.
    async fn record_latency(
        &self,
        ctx: &Context,
        source: ServerLatencyType,
        status: ServerStatus,
        latency: u64,
    ) -> Result<()> {
        let latency_type = {
            let server = self.server.read().await;

            server.latency_type
        };

        if latency_type != source {
            return Ok(());
        }

        self.add_latency(ctx.clone(), status, latency).await
    }

    pub async fn query_server(&self, ctx: Context) -> Result<QueryResponse> {
        let (ip, query_port, addr, query_type, timeout) = {
            let server = self.server.read().await;

            (
                server.ip.clone(),
                server.query_port(),
                server.to_addr(),
                server.query_type,
                server.query_timeout,
            )
        };

        let args = &ctx.args;

        let query_monitor = args
            .parse_query_monitor()
            .ok_or_else(|| anyhow!("Failed to parse query monitor"))?;

        let monitor_only = args.use_query_monitor_only;

        // Format tag.
        let tag = format!("{} ({}:{})", addr, query_type, query_monitor.to_str());

        log_debug!(ctx, "{}: Querying server on port {}...", tag, query_port);

        let mut query = Query::from_srv_type(&query_type)
            .await
            .map_err(|e| anyhow!("Failed to create query: {}", e))?;

        // When we only monitor one query type there is no reason to send the others.
        let (info, users, vars) = if monitor_only {
            match query_monitor {
                QueryMonitor::Info => (
                    Some(query.query_info(&ip, query_port, timeout).await?),
                    None,
                    None,
                ),
                QueryMonitor::Users => (
                    None,
                    Some(query.query_users(&ip, query_port, timeout).await?),
                    None,
                ),
                QueryMonitor::Vars => (
                    None,
                    None,
                    Some(query.query_vars(&ip, query_port, timeout).await?),
                ),
            }
        } else {
            let all = query.query_all(&ip, query_port, timeout).await?;

            (Some(all.info), Some(all.users), Some(all.vars))
        };

        let mut latency_info = 0;
        let mut latency_users = 0;
        let mut latency_vars = 0;

        let mut status = ServerStatus::Unknown;

        if let Some(res) = &info {
            self.apply_info(&ctx, res, query_port).await?;

            latency_info = res.latency;
            status = res.status.clone();
        }

        if let Some(res) = &users {
            self.apply_users(&ctx, res).await?;

            latency_users = res.latency;

            if info.is_none() {
                status = res.status.clone();
            }
        }

        if let Some(res) = &vars {
            self.apply_vars(&ctx, res).await?;

            latency_vars = res.latency;

            if info.is_none() && users.is_none() {
                status = res.status.clone();
            }
        }

        log_debug!(
            ctx,
            "{}: Query finished with status {} (info: {:.2}ms, users: {:.2}ms, vars: {:.2}ms).",
            tag,
            status,
            latency_info as f64 / 1000.0,
            latency_users as f64 / 1000.0,
            latency_vars as f64 / 1000.0
        );

        if args.basic {
            let latency_to_use = match query_monitor {
                QueryMonitor::Info => latency_info,
                QueryMonitor::Users => latency_users,
                QueryMonitor::Vars => latency_vars,
            };

            self.print_query_info(latency_to_use as f64 / 1000.0, &query_monitor)
                .await;
        }

        Ok(QueryResponse {
            status,
            latency_info,
            latency_users,
            latency_vars,
        })
    }
}
