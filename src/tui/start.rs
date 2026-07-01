use std::time::Duration;

use anyhow::Result;
use tokio::task;
use tokio::time::sleep;

use crate::context::Context;
use crate::log_info;
use crate::{log_error, tui::types::Tui};

use crate::logger::level::LogLevel;

impl Tui {
    pub async fn start(ctx: Context) -> Result<()> {
        let ctx_job = ctx.clone();

        // Now create a task to draw the TUI interface at the configured interval.
        let interval = ctx.settings.read().await.tui_draw_interval;

        task::spawn(async move {
            let ctx = ctx_job.clone();

            loop {
                let mut tui = ctx.tui.write().await;

                if tui.draw_cancel_token.is_cancelled() {
                    log_info!(
                        ctx_job.logger.write().await,
                        "TUI interface draw job cancelled, exiting loop"
                    );

                    break;
                }

                if let Err(e) = tui.draw().await {
                    log_error!(
                        ctx_job.logger.write().await,
                        "Failed to start TUI interface draw job: {}",
                        e
                    );
                }

                // Sleep for the configured interval before starting the next draw job.
                sleep(Duration::from_millis(interval)).await;
            }
        });

        Ok(())
    }
}
