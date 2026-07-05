use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, job::job_data::Uuid};

use crate::{context::Context, log_error, log_trace, logger::level::LogLevel, server::ServerCtx};

impl ServerCtx {
    pub async fn setup_task_latency(self: Arc<Self>, ctx: Context) -> Result<Uuid> {
        // Clone the server context for the job closure.
        let job_ctx = ctx.clone();
        let job_self = self.clone();

        let interval = {
            let server = self.server.read().await;

            server.latency_interval.unwrap_or(server.query_interval)
        };

        let lock = Arc::new(Mutex::new(()));

        let job = Job::new_repeated_async(Duration::from_millis(interval), move |_uuid, _l| {
            let self_clone = job_self.clone();
            let ctx = job_ctx.clone();

            let lock = lock.clone();

            Box::pin(async move {
                let ctx = ctx.clone();

                let self_clone = self_clone.clone();

                let (addr, id) = {
                    let server = self_clone.server.read().await;

                    let addr = format!("{}:{}", server.ip, server.port);

                    let id = self_clone.id.clone();

                    (addr, id)
                };

                // Ensure task isn't already running.
                let _guard = match lock.try_lock_owned() {
                    Ok(guard) => guard,
                    Err(_) => {
                        log_trace!(
                            ctx.logger.write().await,
                            "Latency task for server {} is already running, skipping...",
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
