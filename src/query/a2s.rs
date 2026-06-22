use std::time::Duration;

use a2s::{A2SClient, info::ServerType};
use anyhow::{Result, anyhow, bail};

use crate::{
    query::{
        ext::QueryExt,
        types::a2s::{QueryA2sCtx, QueryA2sStatusCodes},
    },
    server::types::{Server, data::ServerStatus},
    time::get_now_ms,
};

use a2s::errors::Error as A2sError;

fn parse_a2s_error(server: &mut Server, e: A2sError) -> Result<()> {
    // Check for standard timeout indicating server is offline.
    if matches!(e, A2sError::ErrTimeout) {
        server.data.status = ServerStatus::Offline;

        return Ok(());
    }

    match e {
        A2sError::Io(_) => {
            server.data.status = ServerStatus::Error;

            server.data.status_code = Some(QueryA2sStatusCodes::IoError as u16);
        }
        A2sError::TryReserveError(_) => {
            server.data.status = ServerStatus::Error;

            server.data.status_code = Some(QueryA2sStatusCodes::TryReserveError as u16);
        }
        A2sError::InvalidResponse => {
            server.data.status = ServerStatus::Error;

            server.data.status_code = Some(QueryA2sStatusCodes::InvalidResponse as u16);
        }
        A2sError::MismatchID => {
            server.data.status = ServerStatus::Error;

            server.data.status_code = Some(QueryA2sStatusCodes::MismatchId as u16);
        }
        A2sError::InvalidBz2Size => {
            server.data.status = ServerStatus::Error;

            server.data.status_code = Some(QueryA2sStatusCodes::InvalidBz2Size as u16);
        }
        A2sError::CheckSumMismatch => {
            server.data.status = ServerStatus::Error;

            server.data.status_code = Some(QueryA2sStatusCodes::ChecksumMismatch as u16);
        }
        _ => {
            server.data.status = ServerStatus::Error;

            server.data.status_code = Some(QueryA2sStatusCodes::Other as u16);
        }
    }

    Ok(())
}

impl QueryExt for QueryA2sCtx {
    async fn init(&mut self) -> Result<()> {
        self.cl = A2SClient::new()
            .await
            .map_err(|e| anyhow!("Failed to initialize A2S client: {}", e))?;

        Ok(())
    }

    async fn query_info(&mut self, server: &mut Server, timeout: u64) -> Result<u64> {
        let data = &mut server.data;

        // Format address.
        let addr = format!("{}:{}", server.ip, server.port_query.unwrap_or(server.port));

        // Set timeout.
        self.cl
            .set_timeout(Duration::from_millis(timeout))
            .map_err(|e| anyhow!("Failed to set A2S timeout: {}", e))?;

        let start = get_now_ms();

        // Query server info.
        let info = match self.cl.info(&addr).await {
            Ok(info) => info,
            Err(e) => match parse_a2s_error(server, e) {
                Ok(_) => return Ok(0),
                Err(e) => bail!("Failed to parse A2S error: {}", e),
            },
        };

        let end = get_now_ms();
        let latency = (end - start) as u64;

        // Update server data.
        data.server_name = Some(info.name);
        data.map_name = Some(info.map);
        data.game_name = Some(info.game);

        data.users_cur = info.players as u16;
        data.users_max = info.max_players as u16;
        data.bots_cur = Some(info.bots as u16);

        data.os = Some(info.server_os.into());

        data.is_secure = info.vac;
        data.is_dedicated = match info.server_type {
            ServerType::Dedicated => true,
            _ => false,
        };
        data.is_public = info.visibility;

        data.version = Some(info.version);

        Ok(latency)
    }

    async fn query_users(&mut self, server: &mut Server, timeout: u64) -> Result<u64> {
        let data = &mut server.data;

        // Reset current list.
        data.users = Vec::new();

        let addr = format!("{}:{}", server.ip, server.port_query.unwrap_or(server.port));

        let start = get_now_ms();

        // Set timeout on the client.
        self.cl
            .set_timeout(Duration::from_millis(timeout))
            .map_err(|e| anyhow!("Failed to set A2S timeout: {}", e))?;

        let users = match self.cl.players(&addr).await {
            Ok(users) => users,
            Err(e) => match parse_a2s_error(server, e) {
                Ok(_) => return Ok(0),
                Err(e) => bail!("Failed to parse A2S error: {}", e),
            },
        };

        let end = get_now_ms();
        let latency = (end - start) as u64;

        for user in users {
            data.users.push(user.into());
        }

        Ok(latency)
    }

    async fn query_vars(&mut self, server: &mut Server, timeout: u64) -> Result<u64> {
        let data = &mut server.data;

        // Reset current list.
        data.vars = Vec::new();

        let addr = format!("{}:{}", server.ip, server.port_query.unwrap_or(server.port));

        // Set timeout on the client.
        self.cl
            .set_timeout(Duration::from_millis(timeout))
            .map_err(|e| anyhow!("Failed to set A2S timeout: {}", e))?;

        let start = get_now_ms();

        let vars = match self.cl.rules(&addr).await {
            Ok(vars) => vars,
            Err(e) => match parse_a2s_error(server, e) {
                Ok(_) => return Ok(0),
                Err(e) => bail!("Failed to parse A2S error: {}", e),
            },
        };

        let end = get_now_ms();

        let latency = (end - start) as u64;

        for var in vars {
            data.vars.push(var.into());
        }

        Ok(latency)
    }
}
