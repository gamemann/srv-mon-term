use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use tokio_cron_scheduler::Job;

use crate::context::Context;
use crate::{log_error, tui::types::Tui};

use crate::logger::level::LogLevel;

impl Tui {
    pub async fn start(ctx: Context) -> Result<u128> {
        let ctx_job = ctx.clone();

        // Now create a task to draw the TUI interface at the configured interval.
        let task_id = {
            let interval = ctx.settings.read().await.tui_draw_interval;

            let job = Job::new_repeated_async(Duration::from_millis(interval), move |_uuid, _l| {
                let ctx = ctx_job.clone();

                Box::pin(async move {
                    match ctx.tui.write().await.draw().await {
                        Ok(_) => (),
                        Err(e) => {
                            log_error!(
                                ctx.logger.write().await,
                                "Failed to draw TUI interface: {}",
                                e
                            );
                        }
                    }
                })
            })
            .map_err(|e| anyhow!("Failed to create TUI interface job: {}", e))?;

            {
                let sch = ctx.sch.read().await;

                match sch.add(job.clone()).await {
                    Ok(_) => (),
                    Err(e) => bail!("Failed to add TUI interface job to scheduler: {}", e),
                };
            }

            job.guid().as_u128()
        };

        Ok(task_id)
    }
}
