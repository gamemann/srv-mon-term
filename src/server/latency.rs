use std::net::{IpAddr, Ipv4Addr};

use anyhow::{Result, bail};
use surge_ping::ping;

use crate::context::Context;
use crate::query::types::Query;
use crate::server::ServerCtx;
use crate::server::data::ServerStatus;
use crate::server::types::latency::{ServerLatency, ServerLatencyType};
use crate::server::types::query::ServerQueryType;

use crate::query::ext::QueryExt;

impl ServerCtx {
    pub async fn run_custom_latency(&self, ctx: Context) -> Result<()> {
        let (latency_type, query_timeout) = {
            let server = self.server.read().await;

            (server.latency_type.clone(), server.query_timeout.clone())
        };

        match latency_type {
            ServerLatencyType::Icmp => {
                // Create payload.
                let payload = vec![0; 1];

                // Parse IP.
                let ip = {
                    let server = self.server.read().await;

                    match server.ip.parse::<Ipv4Addr>() {
                        Ok(ip) => ip,
                        Err(e) => bail!("Failed to parse IP address: {}", e),
                    }
                };

                // Perform ICMP ping.
                let (_, duration) = match ping(IpAddr::V4(ip), &payload).await {
                    Ok(res) => res,
                    Err(e) => bail!("Failed to perform ICMP ping: {}", e),
                };

                let latency = duration.as_millis() as u64;

                self.add_latency(ctx, latency).await
            }
            ServerLatencyType::A2sInfo
            | ServerLatencyType::A2sPlayers
            | ServerLatencyType::A2sRules => {
                // Create the query.
                let mut query = match Query::from_srv_type(&ServerQueryType::A2s).await {
                    Ok(q) => q,
                    Err(e) => bail!("Failed to create query for server: {}", e),
                };

                // Check A2S query type and perform query based off of that.
                let latency = match latency_type {
                    ServerLatencyType::A2sInfo => {
                        let (ip, port, timeout) = {
                            let server = self.server.read().await;

                            (server.ip.clone(), server.port, query_timeout.clone())
                        };

                        let res = match query.query_info(&ip, port, timeout).await {
                            Ok(v) => v,
                            Err(e) => bail!("Failed to query server info: {}", e),
                        };

                        res.latency
                    }
                    ServerLatencyType::A2sPlayers => {
                        let (ip, port, timeout) = {
                            let server = self.server.read().await;

                            (server.ip.clone(), server.port, query_timeout.clone())
                        };

                        let res = match query.query_users(&ip, port, timeout).await {
                            Ok(v) => v,
                            Err(e) => bail!("Failed to query server users: {}", e),
                        };

                        res.latency
                    }
                    ServerLatencyType::A2sRules => {
                        let (ip, port, timeout) = {
                            let server = self.server.read().await;

                            (server.ip.clone(), server.port, query_timeout.clone())
                        };

                        let res = match query.query_vars(&ip, port, timeout).await {
                            Ok(v) => v,
                            Err(e) => bail!("Failed to query server vars: {}", e),
                        };

                        res.latency
                    }
                    _ => 0, // Should NOTTT get here lol
                };

                self.add_latency(ctx, latency).await
            }

            ServerLatencyType::SelfInfo
            | ServerLatencyType::SelfUsers
            | ServerLatencyType::SelfVars => Ok(()),
        }
    }

    pub async fn get_latency(&self, _ctx: Context) -> Option<u64> {
        let latencies = self.latency.read().await;

        latencies.back().map(|l| l.val.clone())
    }

    pub async fn add_latency(&self, _ctx: Context, latency: u64) -> Result<()> {
        let (status, latency_type, latency_history_size) = {
            let server = self.server.read().await;

            (
                server.data.status.clone(),
                server.latency_type.clone(),
                server.latency_history_size,
            )
        };

        let mut latencies = self.latency.write().await;

        latencies.push_back(ServerLatency {
            online: status == ServerStatus::Online,
            type_: latency_type,
            ts: chrono::Utc::now().timestamp_millis() as u64,
            val: latency,
        });

        if latencies.len() > latency_history_size {
            latencies.pop_front();
        }

        Ok(())
    }
}
