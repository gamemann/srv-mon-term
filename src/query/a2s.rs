use std::time::Duration;

use a2s::{A2SClient, info::ServerType};
use anyhow::{Result, anyhow};
use tokio::time::Instant;

use crate::{
    query::{
        ext::QueryExt,
        types::{
            a2s::{QueryA2sCtx, QueryA2sStatusCodes},
            ext::{InfoResponse, QueryResponse, UsersResponse, VarsResponse},
        },
    },
    server::types::data::ServerStatus,
};

use a2s::errors::Error as A2sError;

fn parse_a2s_error(e: A2sError) -> ServerStatus {
    // Check for standard timeout indicating server is offline.
    if matches!(e, A2sError::ErrTimeout) {
        return ServerStatus::Offline;
    }

    let code = match e {
        A2sError::Io(_) => QueryA2sStatusCodes::IoError as u16,
        A2sError::TryReserveError(_) => QueryA2sStatusCodes::TryReserveError as u16,
        A2sError::InvalidResponse => QueryA2sStatusCodes::InvalidResponse as u16,
        A2sError::MismatchID => QueryA2sStatusCodes::MismatchId as u16,
        A2sError::InvalidBz2Size => QueryA2sStatusCodes::InvalidBz2Size as u16,
        A2sError::CheckSumMismatch => QueryA2sStatusCodes::ChecksumMismatch as u16,
        _ => QueryA2sStatusCodes::Other as u16,
    };

    ServerStatus::Error(code)
}

impl QueryA2sCtx {
    /// Applies the per-query timeout to the underlying client.
    fn apply_timeout(&mut self, timeout: u64) -> Result<()> {
        self.cl
            .set_timeout(Duration::from_millis(timeout))
            .map_err(|e| anyhow!("Failed to set A2S timeout: {}", e))?;

        Ok(())
    }
}

impl QueryExt for QueryA2sCtx {
    async fn init(&mut self) -> Result<()> {
        self.cl = A2SClient::new()
            .await
            .map_err(|e| anyhow!("Failed to initialize A2S client: {}", e))?;

        Ok(())
    }

    async fn query_info(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<InfoResponse>> {
        let mut res = QueryResponse::<InfoResponse>::default();

        let addr = format!("{}:{}", ip, port);

        self.apply_timeout(timeout)?;

        let start = Instant::now();

        let info = match self.cl.info(&addr).await {
            Ok(info) => info,
            Err(e) => {
                res.status = parse_a2s_error(e);

                return Ok(res);
            }
        };

        res.status = ServerStatus::Online;
        res.latency = start.elapsed().as_micros() as u64;

        res.data = InfoResponse {
            srv_name: Some(info.name),
            map_name: Some(info.map),
            game_name: Some(info.game),
            game_dir: Some(info.folder),
            game_id: Some(info.app_id),
            game_port: info.extended_server_info.port,
            users_cnt: info.players as u16,
            users_max: info.max_players as u16,
            bots_cnt: Some(info.bots as u16),
            os: Some(info.server_os.into()),
            is_secure: info.vac,
            is_dedicated: matches!(info.server_type, ServerType::Dedicated),
            // A2S reports whether a password is required, which is the inverse of public.
            is_public: !info.visibility,
            version: Some(info.version),
        };

        Ok(res)
    }

    async fn query_users(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<UsersResponse>> {
        let mut res = QueryResponse::<UsersResponse>::default();

        let addr = format!("{}:{}", ip, port);

        self.apply_timeout(timeout)?;

        let start = Instant::now();

        let users = match self.cl.players(&addr).await {
            Ok(users) => users,
            Err(e) => {
                res.status = parse_a2s_error(e);

                return Ok(res);
            }
        };

        res.latency = start.elapsed().as_micros() as u64;
        res.status = ServerStatus::Online;
        res.data.users = users.into_iter().map(|user| user.into()).collect();

        Ok(res)
    }

    async fn query_vars(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<VarsResponse>> {
        let mut res = QueryResponse::<VarsResponse>::default();

        let addr = format!("{}:{}", ip, port);

        self.apply_timeout(timeout)?;

        let start = Instant::now();

        let vars = match self.cl.rules(&addr).await {
            Ok(vars) => vars,
            Err(e) => {
                res.status = parse_a2s_error(e);

                return Ok(res);
            }
        };

        res.latency = start.elapsed().as_micros() as u64;
        res.status = ServerStatus::Online;

        res.data.vars = vars
            .into_iter()
            .filter(|var| !var.name.is_empty() && !var.value.is_empty())
            .map(|var| var.into())
            .collect();

        Ok(res)
    }
}
