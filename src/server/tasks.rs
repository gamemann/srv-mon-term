pub mod latency;
pub mod query;

use std::sync::Arc;

use anyhow::{Result, anyhow};
use tokio_cron_scheduler::job::job_data::Uuid;

use crate::{context::Context, server::ServerCtx};

impl ServerCtx {
    pub async fn setup_tasks(self: Arc<Self>, ctx: Context) -> Result<()> {
        let query_self = self.clone();
        let latency_self = self.clone();

        // Setup query task.
        let query_task_id = query_self
            .setup_task_query(ctx.clone())
            .await
            .map_err(|e| anyhow!("Failed to setup query task: {}", e))?;

        // Setup latency task.
        let latency_task_id = latency_self
            .setup_task_latency(ctx.clone())
            .await
            .map_err(|e| anyhow!("Failed to setup latency task: {}", e))?;

        {
            // Assign task IDs to server context so that we can reference them later.
            let mut tasks = self.tasks.write().await;

            tasks.query_task_id = query_task_id.as_u128().into();
            tasks.latency_task_id = latency_task_id.as_u128().into();
        }

        Ok(())
    }

    pub async fn shutdown_tasks(&self, ctx: Context) -> Result<()> {
        let tasks = self.tasks.read().await;

        if let Some(query_task_id) = tasks.query_task_id {
            let task: Uuid = Uuid::from_u128(query_task_id);

            let sch = ctx.sch.read().await;

            sch.remove(&task.into())
                .await
                .map_err(|e| anyhow!("Failed to remove query job from scheduler: {}", e))?;
        }

        if let Some(latency_task_id) = tasks.latency_task_id {
            let task: Uuid = Uuid::from_u128(latency_task_id);

            let sch = ctx.sch.read().await;

            sch.remove(&task.into())
                .await
                .map_err(|e| anyhow!("Failed to remove latency job from scheduler: {}", e))?;
        }

        Ok(())
    }
}
