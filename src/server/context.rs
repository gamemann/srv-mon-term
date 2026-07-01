use std::collections::VecDeque;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::server::types::{Server, latency::ServerLatency, tasks::ServerTasks};

use std::sync::Arc;

use anyhow::{Result, anyhow, bail};

use crate::context::Context;

pub struct ServerCtx {
    pub id: String,
    pub server: RwLock<Server>,

    pub tasks: RwLock<ServerTasks>,
    pub latency: RwLock<VecDeque<ServerLatency>>,
}

impl ServerCtx {
    pub fn new(id: Option<String>, ip: String, port: u16, port_query: Option<u16>) -> Self {
        Self {
            id: id.unwrap_or_else(|| Uuid::now_v7().to_string()),
            server: RwLock::new(Server::new(ip, port, port_query)),
            tasks: RwLock::new(ServerTasks::default()),
            latency: RwLock::new(VecDeque::new()),
        }
    }

    pub fn from_addr(addr: &str) -> Result<Self> {
        let mut parts = addr.split(':');

        let ip = parts
            .next()
            .ok_or_else(|| anyhow!("Malformed address"))?
            .to_string();

        let port_str = parts
            .next()
            .ok_or_else(|| anyhow!("Malformed address: missing port"))?;

        let port = port_str
            .parse()
            .map_err(|_| anyhow!("Malformed address: invalid port"))?;

        Ok(Self {
            id: Uuid::now_v7().to_string(),
            server: RwLock::new(Server::new(ip, port, None)),
            tasks: RwLock::new(ServerTasks::default()),
            latency: RwLock::new(VecDeque::new()),
        })
    }

    pub async fn get_server_ctx(ctx: Context, id: &str) -> Result<Arc<Self>> {
        let servers = ctx.servers.read().await;

        servers
            .iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or_else(|| anyhow!("Failed to find server context"))
    }

    pub async fn get_server_ctx_by_id(ctx: Context, id: &str) -> Result<Arc<Self>> {
        let servers = ctx.servers.read().await;

        let srv_ctx = servers
            .iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or_else(|| anyhow!("Failed to find server context"))?;

        Ok(srv_ctx)
    }

    pub async fn get_server_ctx_by_addr(ctx: Context, ip: &str, port: u16) -> Result<Arc<Self>> {
        let servers = ctx.servers.read().await;

        let mut srv_ctx = None;

        for s in servers.iter() {
            let srv = s.server.read().await;

            if srv.ip == ip && srv.port == port {
                srv_ctx = Some(s);

                break;
            }
        }

        let srv_ctx = srv_ctx.ok_or_else(|| anyhow!("Failed to find server context"))?;

        Ok(srv_ctx.clone())
    }

    pub async fn remove_from_ctx(ctx: Context, id: &str) -> Result<()> {
        let my_server_idx = (|| async {
            let servers = ctx.servers.read().await;

            for (index, server) in servers.iter().enumerate() {
                if id == server.id {
                    return Some(index);
                }
            }

            None
        })()
        .await;

        let my_server_idx =
            my_server_idx.ok_or_else(|| anyhow!("Failed to find server in context"))?;

        {
            let mut servers = ctx.servers.write().await;

            servers.remove(my_server_idx);
        }

        Ok(())
    }

    pub async fn add(self: Arc<Self>, ctx: Context) -> Result<()> {
        {
            let mut servers = ctx.servers.write().await;

            // Make sure the server doesn't already exist in the context.
            if servers.iter().any(|s| s.id == self.id) {
                return Err(anyhow!(
                    "Server with ID {} already exists in context",
                    self.id
                ));
            }

            servers.push(self.clone());
        }

        Ok(())
    }

    pub async fn delete(&mut self, ctx: Context) -> Result<()> {
        {
            let mut servers = ctx.servers.write().await;

            // Make sure the server exists in the context.
            if !servers.iter().any(|s| s.id == self.id) {
                return Err(anyhow!(
                    "Server with ID {} does not exist in context",
                    self.id
                ));
            }

            let my_server_idx = servers
                .iter()
                .position(|s| s.id == self.id)
                .ok_or_else(|| anyhow!("Failed to find server in context"))?;

            servers.remove(my_server_idx);
        }

        // Shut down and remove tasks for the server.
        match self.shutdown_tasks(ctx.clone()).await {
            Ok(_) => (),
            Err(e) => bail!("Failed to shutdown tasks for server: {}", e),
        }

        // Remove the server from the main context.
        Self::remove_from_ctx(ctx.clone(), &self.id)
            .await
            .map_err(|e| anyhow!("Failed to remove server from context: {}", e))?;

        Ok(())
    }
}
