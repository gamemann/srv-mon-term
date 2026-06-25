use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow, bail};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, job::job_data::Uuid};

use crate::{
    context::Context,
    log_debug, log_error, log_trace,
    logger::level::LogLevel,
    server::{
        ServerCtx,
        data::ServerStatus,
        types::latency::{ServerLatency, ServerLatencyType},
    },
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

            let mut sch = ctx.sch.write().await;

            let query_lock = Arc::new(Mutex::new(()));

            let latency_info_history = Arc::new(Mutex::new(Vec::new()));
            let latency_users_history = Arc::new(Mutex::new(Vec::new()));
            let latency_vars_history = Arc::new(Mutex::new(Vec::new()));

            // Create query task.
            let query_latency_info_history = latency_info_history.clone();
            let query_latency_users_history = latency_users_history.clone();
            let query_latency_vars_history = latency_vars_history.clone();

            let query_task_id = match sch
                .add(
                    Job::new_repeated_async(Duration::from_millis(interval), move |_uuid, _l| {
                        let self_clone = query_self.clone();
                        let ctx = query_ctx.clone();

                        let lock = query_lock.clone();

                        let latency_info_history = query_latency_info_history.clone();
                        let latency_users_history = query_latency_users_history.clone();
                        let latency_vars_history = query_latency_vars_history.clone();

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
                            let _guard = match lock.try_lock_owned() {
                                Ok(guard) => guard,
                                Err(_) => {
                                    log_trace!(
                                        ctx.logger.write().await,
                                        "Query task for server {} is already running, skipping...",
                                        addr
                                    );

                                    return;
                                }
                            };

                            log_debug!(
                                ctx.logger.write().await,
                                "Running query task for server '{}'...",
                                addr
                            );

                            let res = match self_clone.query_server(ctx.clone()).await {
                                Ok(res) => res,
                                Err(e) => {
                                    let id = id.clone();

                                    log_error!(
                                        ctx.logger.write().await,
                                        "Failed to query server {} ({}): {}",
                                        id,
                                        addr,
                                        e
                                    );

                                    return;
                                }
                            };

                            // Spawn task to add latency history to avoid blocking the query task.
                            tokio::spawn(async move {
                                let now = chrono::Utc::now().timestamp() as u64;

                                // Add info latency history.
                                {
                                    let mut info_history = latency_info_history.lock().await;

                                    let latency = ServerLatency {
                                        online: res.status == ServerStatus::Online,
                                        type_: ServerLatencyType::SelfInfo,
                                        ts: now,
                                        val: res.latency_info,
                                    };

                                    info_history.push(latency);
                                }

                                // Add users latency history.
                                {
                                    let mut users_history = latency_users_history.lock().await;

                                    let latency = ServerLatency {
                                        online: res.status == ServerStatus::Online,
                                        type_: ServerLatencyType::SelfUsers,
                                        ts: now,
                                        val: res.latency_users,
                                    };

                                    users_history.push(latency);
                                }

                                // Add vars latency history.
                                {
                                    let mut vars_history = latency_vars_history.lock().await;

                                    let latency = ServerLatency {
                                        online: res.status == ServerStatus::Online,
                                        type_: ServerLatencyType::SelfVars,
                                        ts: now,
                                        val: res.latency_vars,
                                    };

                                    vars_history.push(latency);
                                }
                            });
                        })
                    })
                    .map_err(|e| anyhow!("Failed to create job: {}", e))?,
                )
                .await
            {
                Ok(q) => Some(q),
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
                    .map_err(|e| anyhow!("Failed to create job: {}", e))?,
                )
                .await
            {
                Ok(q) => Some(q),
                Err(e) => bail!("Failed to add job to scheduler: {}", e),
            };

            // Clone things we need for the shutdown handler.
            let shutdown_ctx = ctx.clone();

            let shutdown_latency_info_history = latency_info_history.clone();
            let shutdown_latency_users_history = latency_users_history.clone();
            let shutdown_latency_vars_history = latency_vars_history.clone();

            sch.set_shutdown_handler(Box::new(move || {
                let ctx = shutdown_ctx.clone();

                let latency_info_history = shutdown_latency_info_history.clone();
                let latency_users_history = shutdown_latency_users_history.clone();
                let latency_vars_history = shutdown_latency_vars_history.clone();

                Box::pin(async move {
                    if ctx.args.basic {
                        // Retrieve minimum, maximum and average latency for each type and log it.

                        let (info_min, info_max, info_avg) = {
                            let history = latency_info_history.lock().await;

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
                            let history = latency_users_history.lock().await;

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
                            let history = latency_vars_history.lock().await;

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

            {
                // Assign task IDs to server context.
                let mut tasks = self.tasks.write().await;

                tasks.query_task_id = query_task_id.and_then(|id| Some(id.as_u128()));
                tasks.latency_task_id = latency_task_id.and_then(|id| Some(id.as_u128()));
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
