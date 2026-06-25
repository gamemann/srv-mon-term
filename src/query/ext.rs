use anyhow::Result;

use crate::query::types::{
    Query,
    ext::{InfoResponse, QueryResponse, UsersResponse, VarsResponse},
};

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
}

impl QueryExt for Query {
    async fn init(&mut self) -> Result<()> {
        match self {
            Query::A2s(ctx) => ctx.init().await,
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
        }
    }
}
