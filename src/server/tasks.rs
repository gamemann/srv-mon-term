use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow, bail};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, job::job_data::Uuid};

use crate::{
    context::Context, log_debug, log_error, log_trace, logger::level::LogLevel, server::ServerCtx,
};

impl ServerCtx {
    pub async fn setup_tasks(self: Arc<Self>, ctx: Context) -> Result<()> {
        // Clone self and context for use in tasks.
        let query_self = self.clone();
        let query_ctx = ctx.clone();

        let latency_self = self.clone();
        let latency_ctx = ctx.clone();

        {
            let interval = {
                let server = self.server.read().await;

                server.query_interval
            };

            let sch = ctx.sch.read().await;

            let query_lock = Arc::new(Mutex::new(()));

            // Create query task.
            let query_task_id = match sch
                .add(
                    Job::new_repeated_async(Duration::from_millis(interval), move |_uuid, _l| {
                        let self_clone = query_self.clone();
                        let ctx = query_ctx.clone();

                        let lock = query_lock.clone();

                        Box::pin(async move {
                            let ctx = ctx.clone();

                            let self_clone = self_clone.clone();

                            let (addr, id) = {
                                let server = self_clone.server.read().await;

                                let addr = format!("{}:{}", server.ip.clone(), server.port);

                                let id = server.id.clone().unwrap_or("N/A".to_string());

                                (addr, id)
                            };

                            // Ensure task isn't already running.
                            match lock.try_lock_owned() {
                                Ok(_) => (),
                                Err(_) => {
                                    log_trace!(
                                        ctx.logger.write().await,
                                        "Query task for server {} is already running, skipping...",
                                        addr
                                    );

                                    return;
                                }
                            }

                            log_debug!(
                                ctx.logger.write().await,
                                "Running query task for server '{}'...",
                                addr
                            );

                            match self_clone.query_server(ctx.clone()).await {
                                Ok(_) => (),
                                Err(e) => {
                                    let id = id.clone();

                                    log_error!(
                                        ctx.logger.write().await,
                                        "Failed to query server {} ({}): {}",
                                        id,
                                        addr,
                                        e
                                    );
                                }
                            }
                        })
                    })
                    .map_err(|e| anyhow!("Failed to create job: {}", e))?,
                )
                .await
            {
                Ok(q) => Some(q.as_u128()),
                Err(e) => bail!("Failed to add job to scheduler: {}", e),
            };

            let interval = {
                let server = self.server.read().await;

                server.latency_interval.unwrap_or(server.query_interval)
            };

            let latency_lock = Arc::new(Mutex::new(()));

            // Create latency task.
            let latency_task_id = match sch
                .add(
                    Job::new_repeated_async(Duration::from_millis(interval), move |_uuid, _l| {
                        let self_clone = latency_self.clone();
                        let ctx = latency_ctx.clone();

                        let lock = latency_lock.clone();

                        Box::pin(async move {
                            let ctx = ctx.clone();

                            let self_clone = self_clone.clone();

                            let (addr, id) = {
                                let server = self_clone.server.read().await;

                                let addr = format!("{}:{}", server.ip, server.port);

                                let id = server.id.clone().unwrap_or("N/A".to_string());

                                (addr, id)
                            };

                            // Ensure task isn't already running.
                            match lock.try_lock_owned() {
                                Ok(_) => (),
                                Err(_) => {
                                    log_trace!(
                                        ctx.logger.write().await,
                                        "{}: Latency task is already running, skipping...",
                                        addr
                                    );

                                    return;
                                }
                            };

                            match self_clone.run_custom_latency(ctx.clone()).await {
                                Ok(_) => (),
                                Err(e) => {
                                    let id = id.clone();

                                    log_error!(
                                        ctx.logger.write().await,
                                        "Failed to run latency check for server {} ({}): {}",
                                        id,
                                        addr,
                                        e
                                    );
                                }
                            }
                        })
                    })
                    .map_err(|e| anyhow!("Failed to create job: {}", e))?,
                )
                .await
            {
                Ok(q) => Some(q.as_u128()),
                Err(e) => bail!("Failed to add job to scheduler: {}", e),
            };

            {
                // Assign task IDs to server context.
                let mut tasks = self.tasks.write().await;

                tasks.query_task_id = query_task_id;
                tasks.latency_task_id = latency_task_id;
            }
        }
        Ok(())
    }

    pub async fn shutdown_tasks(&self, ctx: Context) -> Result<()> {
        let sch = ctx.sch.read().await;

        let tasks = self.tasks.read().await;

        if let Some(query_task_id) = tasks.query_task_id {
            let task: Uuid = Uuid::from_u128(query_task_id);

            sch.remove(&task.into())
                .await
                .map_err(|e| anyhow!("Failed to remove query job from scheduler: {}", e))?;
        }

        if let Some(latency_task_id) = tasks.latency_task_id {
            let task: Uuid = Uuid::from_u128(latency_task_id);

            sch.remove(&task.into())
                .await
                .map_err(|e| anyhow!("Failed to remove latency job from scheduler: {}", e))?;
        }

        Ok(())
    }
}
