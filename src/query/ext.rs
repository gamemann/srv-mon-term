use anyhow::Result;

use crate::{query::types::Query, server::types::Server};

#[allow(async_fn_in_trait)]
pub trait QueryExt {
    async fn init(&mut self) -> Result<()>;

    async fn query_info(
        &mut self,
        server: &mut Server,
        timeout: u64,
        set_offline: bool,
    ) -> Result<u64>;
    async fn query_users(
        &mut self,
        server: &mut Server,
        timeout: u64,
        set_offline: bool,
    ) -> Result<u64>;
    async fn query_vars(
        &mut self,
        server: &mut Server,
        timeout: u64,
        set_offline: bool,
    ) -> Result<u64>;
}

impl QueryExt for Query {
    async fn init(&mut self) -> Result<()> {
        match self {
            Query::A2s(ctx) => ctx.init().await,
        }
    }

    async fn query_info(
        &mut self,
        server: &mut Server,
        timeout: u64,
        set_offline: bool,
    ) -> Result<u64> {
        match self {
            Query::A2s(ctx) => ctx.query_info(server, timeout, set_offline).await,
        }
    }

    async fn query_users(
        &mut self,
        server: &mut Server,
        timeout: u64,
        set_offline: bool,
    ) -> Result<u64> {
        match self {
            Query::A2s(ctx) => ctx.query_users(server, timeout, set_offline).await,
        }
    }

    async fn query_vars(
        &mut self,
        server: &mut Server,
        timeout: u64,
        set_offline: bool,
    ) -> Result<u64> {
        match self {
            Query::A2s(ctx) => ctx.query_vars(server, timeout, set_offline).await,
        }
    }
}
