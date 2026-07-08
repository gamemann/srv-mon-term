use anyhow::{Result, anyhow, bail};

use crate::cli::Args;
use crate::context::Context;

use crate::settings::Settings;
use crate::store::ext::StoreExt;

pub async fn store_setup(ctx: Context) -> Result<()> {
    let new_settings = {
        let mut store = ctx.store.write().await;

        match store.init().await {
            Ok(_) => (),
            Err(e) => bail!("Failed to initialize store: {}", e),
        }

        // Let's retrieve our current settings.
        match store.settings_fetch().await {
            Ok(settings) => settings,
            Err(e) => bail!("Failed to fetch settings from store: {}", e),
        }
    };

    // Write settings retrieved from store to the context.
    {
        let mut settings = ctx.settings.write().await;

        *settings = new_settings.clone();
    }

    // Look for overrides from the command line.
    let args = ctx.args.clone();

    if let Some(interval) = args.draw_interval {
        let mut settings = ctx.settings.write().await;

        settings.tui_draw_interval = interval;
    }

    if let Some(interval) = args.input_poll_interval {
        let mut settings = ctx.settings.write().await;

        settings.tui_input_poll_interval = interval;
    }

    if let Some(sz) = args.log_max_buffer_size {
        let mut settings = ctx.settings.write().await;

        settings.log_max_buffer_size = sz;
    }

    if let Some(path) = args.log_path.clone() {
        let mut settings = ctx.settings.write().await;

        settings.log_path = Some(path);
    }

    if let Some(levels) = args.log_levels {
        let mut settings = ctx.settings.write().await;

        settings.log_levels = Args::parse_log_levels(levels);
    }

    // Check if we should save the settings back to the store.
    if args.save {
        let mut store = ctx.store.write().await;

        let settings = ctx.settings.read().await;

        store
            .settings_save(&settings)
            .await
            .map_err(|e| anyhow!("Failed to save settings to store: {}", e))?;
    }

    // List settings now if needed.
    Settings::log_settings(ctx.clone()).await;

    Ok(())
}
