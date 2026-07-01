use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, job::job_data::Uuid};

use crate::{
    context::Context,
    log_error, log_trace,
    logger::level::LogLevel,
    server::{ServerCtx, types::latency::ServerLatencyType},
};

impl ServerCtx {
    pub async fn setup_task_latency(self: Arc<Self>, ctx: Context) -> Result<Uuid> {
        // Clone the server context for the job closure.
        let job_ctx = ctx.clone();
        let job_self = self.clone();

        // Let's lock the scheduler.
        let mut sch = ctx.sch.write().await;

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

        // Clone things we need for the shutdown handler.
        let shutdown_ctx = ctx.clone();
        let shutdown_self = self.clone();

        sch.set_shutdown_handler(Box::new(move || {
            let ctx = shutdown_ctx.clone();
            let self_clone = shutdown_self.clone();

            Box::pin(async move {
                if ctx.args.basic {
                    // Retrieve minimum, maximum and average latency for each type and log it.

                    let (info_min, info_max, info_avg) = {
                        let history = self_clone.latency.read().await;

                        let history = history
                            .iter()
                            .filter(|x| x.type_ == ServerLatencyType::A2sInfo)
                            .collect::<Vec<_>>();

                        let min = history
                            .iter()
                            .min_by_key(|l| l.val)
                            .map(|l| l.val)
                            .unwrap_or(0);

                        let max = history
                            .iter()
                            .max_by_key(|l| l.val)
                            .map(|l| l.val)
                            .unwrap_or(0);

                        let avg = if history.len() > 0 {
                            history.iter().map(|l| l.val).sum::<u64>() / history.len() as u64
                        } else {
                            0
                        };

                        (
                            min as f64 / 1000.0,
                            max as f64 / 1000.0,
                            avg as f64 / 1000.0,
                        )
                    };

                    let (users_min, users_max, users_avg) = {
                        let history = self_clone.latency.read().await;

                        let history = history
                            .iter()
                            .filter(|x| x.type_ == ServerLatencyType::A2sPlayers)
                            .collect::<Vec<_>>();

                        let min = history
                            .iter()
                            .min_by_key(|l| l.val)
                            .map(|l| l.val)
                            .unwrap_or(0);

                        let max = history
                            .iter()
                            .max_by_key(|l| l.val)
                            .map(|l| l.val)
                            .unwrap_or(0);

                        let avg = if history.len() > 0 {
                            history.iter().map(|l| l.val).sum::<u64>() / history.len() as u64
                        } else {
                            0
                        };

                        (
                            min as f64 / 1000.0,
                            max as f64 / 1000.0,
                            avg as f64 / 1000.0,
                        )
                    };

                    let (vars_min, vars_max, vars_avg) = {
                        let history = self_clone.latency.read().await;

                        let history = history
                            .iter()
                            .filter(|x| x.type_ == ServerLatencyType::A2sRules)
                            .collect::<Vec<_>>();

                        let min = history
                            .iter()
                            .min_by_key(|l| l.val)
                            .map(|l| l.val)
                            .unwrap_or(0);

                        let max = history
                            .iter()
                            .max_by_key(|l| l.val)
                            .map(|l| l.val)
                            .unwrap_or(0);

                        let avg = if history.len() > 0 {
                            history.iter().map(|l| l.val).sum::<u64>() / history.len() as u64
                        } else {
                            0
                        };

                        (
                            min as f64 / 1000.0,
                            max as f64 / 1000.0,
                            avg as f64 / 1000.0,
                        )
                    };

                    println!("Latency Summary:");

                    println!(
                        "  Info Latency: min: {:.2} ms, max: {:.2} ms, avg: {:.2} ms",
                        info_min, info_max, info_avg
                    );
                    println!(
                        "  Users Latency: min: {:.2} ms, max: {:.2} ms, avg: {:.2} ms",
                        users_min, users_max, users_avg
                    );
                    println!(
                        "  Vars Latency: min: {:.2} ms, max: {:.2} ms, avg: {:.2} ms",
                        vars_min, vars_max, vars_avg
                    );
                }
            })
        }));

        // Retrieve the job ID before adding it to the scheduler.
        let job_id = job.guid();

        sch.add(job)
            .await
            .map_err(|e| anyhow!("Failed to add job to scheduler: {}", e))?;

        Ok(job_id.into())
    }
}
