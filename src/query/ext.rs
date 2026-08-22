use anyhow::Result;

use crate::{
    query::types::{
        Query,
        ext::{InfoResponse, QueryResponse, UsersResponse, VarsResponse},
    },
    server::data::ServerStatus,
};

/// Every response a single query pass can produce.
pub struct QueryAllResponse {
    pub info: QueryResponse<InfoResponse>,
    pub users: QueryResponse<UsersResponse>,
    pub vars: QueryResponse<VarsResponse>,
}

impl QueryAllResponse {
    /// Builds an empty response where every sub-query carries the same status.
    ///
    /// Used when a protocol answers all three queries from one exchange and that exchange failed.
    pub fn from_status(status: ServerStatus) -> Self {
        Self {
            info: QueryResponse {
                status: status.clone(),
                ..Default::default()
            },
            users: QueryResponse {
                status: status.clone(),
                ..Default::default()
            },
            vars: QueryResponse {
                status,
                ..Default::default()
            },
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait QueryExt {
    async fn init(&mut self) -> Result<()>;

    async fn query_info(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<InfoResponse>>;
    async fn query_users(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<UsersResponse>>;
    async fn query_vars(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<VarsResponse>>;

    /// Runs every query for the server in one pass.
    ///
    /// Protocols that answer info, users and vars from a single exchange override this so we
    /// only hit the server once per interval. The default runs the three queries in order and
    /// gives up early when the server doesn't answer the first one.
    async fn query_all(&mut self, ip: &str, port: u16, timeout: u64) -> Result<QueryAllResponse> {
        let info = self.query_info(ip, port, timeout).await?;

        if info.status != ServerStatus::Online {
            return Ok(QueryAllResponse::from_status(info.status));
        }

        let users = self.query_users(ip, port, timeout).await?;
        let vars = self.query_vars(ip, port, timeout).await?;

        Ok(QueryAllResponse { info, users, vars })
    }
}

impl QueryExt for Query {
    async fn init(&mut self) -> Result<()> {
        match self {
            Query::A2s(ctx) => ctx.init().await,
            Query::Quake3(ctx) => ctx.init().await,
            Query::Minecraft(ctx) => ctx.init().await,
            Query::Bedrock(ctx) => ctx.init().await,
            Query::GameSpy1(ctx) => ctx.init().await,
            Query::GameSpy3(ctx) => ctx.init().await,
            Query::FiveM(ctx) => ctx.init().await,
        }
    }

    async fn query_info(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<InfoResponse>> {
        match self {
            Query::A2s(ctx) => ctx.query_info(ip, port, timeout).await,
            Query::Quake3(ctx) => ctx.query_info(ip, port, timeout).await,
            Query::Minecraft(ctx) => ctx.query_info(ip, port, timeout).await,
            Query::Bedrock(ctx) => ctx.query_info(ip, port, timeout).await,
            Query::GameSpy1(ctx) => ctx.query_info(ip, port, timeout).await,
            Query::GameSpy3(ctx) => ctx.query_info(ip, port, timeout).await,
            Query::FiveM(ctx) => ctx.query_info(ip, port, timeout).await,
        }
    }

    async fn query_users(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<UsersResponse>> {
        match self {
            Query::A2s(ctx) => ctx.query_users(ip, port, timeout).await,
            Query::Quake3(ctx) => ctx.query_users(ip, port, timeout).await,
            Query::Minecraft(ctx) => ctx.query_users(ip, port, timeout).await,
            Query::Bedrock(ctx) => ctx.query_users(ip, port, timeout).await,
            Query::GameSpy1(ctx) => ctx.query_users(ip, port, timeout).await,
            Query::GameSpy3(ctx) => ctx.query_users(ip, port, timeout).await,
            Query::FiveM(ctx) => ctx.query_users(ip, port, timeout).await,
        }
    }

    async fn query_vars(
        &mut self,
        ip: &str,
        port: u16,
        timeout: u64,
    ) -> Result<QueryResponse<VarsResponse>> {
        match self {
            Query::A2s(ctx) => ctx.query_vars(ip, port, timeout).await,
            Query::Quake3(ctx) => ctx.query_vars(ip, port, timeout).await,
            Query::Minecraft(ctx) => ctx.query_vars(ip, port, timeout).await,
            Query::Bedrock(ctx) => ctx.query_vars(ip, port, timeout).await,
            Query::GameSpy1(ctx) => ctx.query_vars(ip, port, timeout).await,
            Query::GameSpy3(ctx) => ctx.query_vars(ip, port, timeout).await,
            Query::FiveM(ctx) => ctx.query_vars(ip, port, timeout).await,
        }
    }

    async fn query_all(&mut self, ip: &str, port: u16, timeout: u64) -> Result<QueryAllResponse> {
        match self {
            Query::A2s(ctx) => ctx.query_all(ip, port, timeout).await,
            Query::Quake3(ctx) => ctx.query_all(ip, port, timeout).await,
            Query::Minecraft(ctx) => ctx.query_all(ip, port, timeout).await,
            Query::Bedrock(ctx) => ctx.query_all(ip, port, timeout).await,
            Query::GameSpy1(ctx) => ctx.query_all(ip, port, timeout).await,
            Query::GameSpy3(ctx) => ctx.query_all(ip, port, timeout).await,
            Query::FiveM(ctx) => ctx.query_all(ip, port, timeout).await,
        }
    }
}
