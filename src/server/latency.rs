use std::time::Duration;

use anyhow::{Result, bail};
use ratatui::style::Color;
use surge_ping::ping;
use tokio::time::timeout;

use crate::context::Context;
use crate::query::proto::net::resolve;
use crate::query::types::Query;
use crate::server::ServerCtx;
use crate::server::data::ServerStatus;
use crate::server::types::latency::{ServerLatency, ServerLatencyType};

/// Aggregated latency values (in milliseconds) over the samples we still hold.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatencySummary {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub samples: usize,
}

use crate::query::ext::QueryExt;

impl ServerCtx {
    /// Runs the latency probe for servers that measure latency outside of the regular query
    /// task (ICMP or a dedicated query).
    pub async fn run_custom_latency(&self, ctx: Context) -> Result<()> {
        let (ip, query_port, query_type, latency_type, latency_timeout) = {
            let server = self.server.read().await;

            (
                server.ip.clone(),
                server.query_port(),
                server.query_type,
                server.latency_type,
                server.latency_timeout.unwrap_or(server.query_timeout),
            )
        };

        match latency_type {
            ServerLatencyType::Icmp => {
                // Create payload.
                let payload = vec![0; 1];

                let addr = resolve(&ip, 0).await?;

                let dur = Duration::from_millis(latency_timeout.max(1));

                // Perform ICMP ping.
                let (_, duration) = match timeout(dur, ping(addr.ip(), &payload)).await {
                    Ok(Ok(res)) => res,
                    Ok(Err(e)) => bail!("Failed to perform ICMP ping: {}", e),
                    Err(_) => {
                        self.add_latency(ctx, ServerStatus::Offline, 0).await?;

                        bail!("ICMP ping timed out after {}ms", latency_timeout);
                    }
                };

                self.add_latency(ctx, ServerStatus::Online, duration.as_micros() as u64)
                    .await
            }

            // These run an extra query using whichever protocol the server is configured with.
            ServerLatencyType::A2sInfo
            | ServerLatencyType::A2sPlayers
            | ServerLatencyType::A2sRules => {
                let mut query = match Query::from_srv_type(&query_type).await {
                    Ok(q) => q,
                    Err(e) => bail!("Failed to create query for server: {}", e),
                };

                let res = match latency_type {
                    ServerLatencyType::A2sPlayers => query
                        .query_users(&ip, query_port, latency_timeout)
                        .await
                        .map(|r| (r.status, r.latency)),
                    ServerLatencyType::A2sRules => query
                        .query_vars(&ip, query_port, latency_timeout)
                        .await
                        .map(|r| (r.status, r.latency)),
                    _ => query
                        .query_info(&ip, query_port, latency_timeout)
                        .await
                        .map(|r| (r.status, r.latency)),
                };

                let (status, latency) = match res {
                    Ok(v) => v,
                    Err(e) => bail!("Failed to query server for latency: {}", e),
                };

                self.add_latency(ctx, status, latency).await
            }

            // Latency is taken from the regular query task for these.
            ServerLatencyType::SelfInfo
            | ServerLatencyType::SelfUsers
            | ServerLatencyType::SelfVars => Ok(()),
        }
    }

    /// Summarises the recorded latency for the given latency sources.
    ///
    /// Returns `None` when we never got a usable sample (server offline the whole time).
    pub async fn latency_summary(&self, types: &[ServerLatencyType]) -> Option<LatencySummary> {
        let history = self.latency.read().await;

        let samples: Vec<u64> = history
            .iter()
            .filter(|l| types.contains(&l.type_) && l.val > 0)
            .map(|l| l.val)
            .collect();

        if samples.is_empty() {
            return None;
        }

        let min = *samples.iter().min()?;
        let max = *samples.iter().max()?;
        let avg = samples.iter().sum::<u64>() / samples.len() as u64;

        Some(LatencySummary {
            min: min as f64 / 1000.0,
            max: max as f64 / 1000.0,
            avg: avg as f64 / 1000.0,
            samples: samples.len(),
        })
    }

    pub async fn get_latency(&self, _ctx: Context) -> Option<u64> {
        let latencies = self.latency.read().await;

        latencies.back().map(|l| l.val)
    }

    pub async fn add_latency(
        &self,
        _ctx: Context,
        status: ServerStatus,
        latency: u64,
    ) -> Result<()> {
        let (latency_type, latency_history_size) = {
            let server = self.server.read().await;

            (server.latency_type, server.latency_history_size.max(1))
        };

        let mut latencies = self.latency.write().await;

        latencies.push_back(ServerLatency {
            online: status == ServerStatus::Online,
            type_: latency_type,
            ts: chrono::Utc::now().timestamp_millis() as u64,
            val: latency,
        });

        while latencies.len() > latency_history_size {
            latencies.pop_front();
        }

        Ok(())
    }
}

pub fn get_latency_color(ms: f64) -> Color {
    if ms < 80.0 {
        Color::Green
    } else if ms < 150.0 {
        Color::Yellow
    } else {
        Color::LightRed
    }
}
