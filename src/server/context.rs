use std::collections::VecDeque;

use tokio::sync::RwLock;

use crate::server::types::{Server, latency::ServerLatency, tasks::ServerTasks};

use std::sync::Arc;

use anyhow::{Result, anyhow, bail};

use crate::context::Context;

use crate::store::ext::StoreExt;

pub struct ServerCtx {
    pub server: RwLock<Server>,

    pub tasks: RwLock<ServerTasks>,
    pub latency: RwLock<VecDeque<ServerLatency>>,
}

impl ServerCtx {
    pub fn new(ip: String, port: u16, port_query: Option<u16>) -> Self {
        Self {
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
            server: RwLock::new(Server::new(ip, port, None)),
            tasks: RwLock::new(ServerTasks::default()),
            latency: RwLock::new(VecDeque::new()),
        })
    }

    pub async fn get_server_ctx(ctx: Context, server: &Server) -> Result<Arc<Self>> {
        let servers = ctx.servers.read().await;

        let mut srv_ctx = None;

        for s in servers.iter() {
            let srv = s.server.read().await;

            if srv.ip == server.ip && srv.port == server.port {
                srv_ctx = Some(s);

                break;
            }
        }

        let srv_ctx = srv_ctx.ok_or_else(|| anyhow!("Failed to find server context"))?;

        Ok(srv_ctx.clone())
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

    pub async fn remove_from_ctx(ctx: Context, server: &ServerCtx) -> Result<()> {
        let (my_id, my_ip, my_port) = {
            let s = server.server.read().await;

            (s.id.clone(), s.ip.clone(), s.port)
        };

        let my_server_idx = (|| async {
            let servers = ctx.servers.read().await;

            for (index, server) in servers.iter().enumerate() {
                let s = server.server.read().await;

                let id = s.id.clone();

                let ip = s.ip.clone();
                let port = s.port;

                if (id.is_some() && id == my_id) || (ip == my_ip && port == my_port) {
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
            let server = self.server.write().await;

            // Attempt to add to store.
            let mut store = ctx.store.write().await;

            match store.srv_add(&server).await {
                Ok(_) => (),
                Err(e) => bail!(
                    "Failed to add server to store ({}): {}",
                    store.get_store_name(),
                    e
                ),
            };
        }

        Ok(())
    }

    pub async fn delete(&mut self, ctx: Context) -> Result<()> {
        {
            // Attempt to remove server from store.
            let mut server = self.server.read().await;

            let mut store = ctx.store.write().await;

            match store.srv_delete(&mut server).await {
                Ok(_) => (),
                Err(e) => bail!(
                    "Failed to remove server from store ({}): {}",
                    store.get_store_name(),
                    e
                ),
            };
        }

        // Shut down and remove tasks for the server.
        match self.shutdown_tasks(ctx.clone()).await {
            Ok(_) => (),
            Err(e) => bail!("Failed to shutdown tasks for server: {}", e),
        }

        // Remove the server from the main context.
        Self::remove_from_ctx(ctx.clone(), self)
            .await
            .map_err(|e| anyhow!("Failed to remove server from context: {}", e))?;

        Ok(())
    }
}
