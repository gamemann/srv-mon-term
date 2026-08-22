use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, job::job_data::Uuid};

use crate::{
    context::Context,
    log_error, log_info, log_trace,
    logger::Logger,
    logger::level::LogLevel,
    server::{ServerCtx, data::ServerStatus, types::latency::ServerLatencyType},
};

impl ServerCtx {
    pub async fn setup_task_latency(self: Arc<Self>, ctx: Context) -> Result<Uuid> {
        // Clone the server context for the job closure.
        let job_ctx = ctx.clone();
        let job_self = self.clone();

        let (interval, latency_type) = {
            let server = self.server.read().await;

            (
                server
                    .latency_interval
                    .filter(|i| *i > 0)
                    .unwrap_or(server.query_interval),
                server.latency_type,
            )
        };

        let lock = Arc::new(Mutex::new(()));

        let job = Job::new_repeated_async(Duration::from_millis(interval), move |_uuid, _l| {
            let self_clone = job_self.clone();
            let ctx = job_ctx.clone();

            let lock = lock.clone();

            Box::pin(async move {
                let ctx = ctx.clone();

                let self_clone = self_clone.clone();

                let addr = {
                    let server = self_clone.server.read().await;

                    let addr = format!("{}:{}", server.ip, server.port);

                    addr
                };

                // Ensure task isn't already running.
                let _guard = match lock.try_lock_owned() {
                    Ok(guard) => guard,
                    Err(_) => {
                        log_trace!(ctx,
                            "Latency task for server {} is already running, skipping...",
                            addr
                        );

                        return;
                    }
                };

                // Nothing to do when latency comes from the regular query task.
                if matches!(
                    latency_type,
                    ServerLatencyType::SelfInfo
                        | ServerLatencyType::SelfUsers
                        | ServerLatencyType::SelfVars
                ) {
                    return;
                }

                match self_clone.run_custom_latency(ctx.clone()).await {
                    Ok(_) => {
                        let mut statuses = self_clone.statuses.write().await;

                        if statuses.latency != ServerStatus::Online {
                            log_info!(ctx,
                                "Latency task for server '{}' completed successfully, setting status to online.",
                                addr
                            );

                            statuses.latency = ServerStatus::Online;
                        }
                    }
                    Err(e) => {
                        log_error!(ctx,
                            "Failed to run latency task for server '{}': {}",
                            addr,
                            e
                        );

                        // Set the latency status to offline since the task has failed.
                        let mut statuses = self_clone.statuses.write().await;
                        statuses.latency = ServerStatus::Offline;
                    }
                }
            })
        })
        .map_err(|e| anyhow!("Failed to create job: {}", e))?;

        // Retrieve the job ID before adding it to the scheduler.
        let job_id = job.guid();

        // Let's lock the scheduler.
        let sch = ctx.sch.read().await;

        sch.add(job)
            .await
            .map_err(|e| anyhow!("Failed to add job to scheduler: {}", e))?;

        Ok(job_id.into())
    }
}
