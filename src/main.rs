#![allow(dead_code)]
#![allow(unused_variables)]

use std::process;

use clap::Parser;
use gmon::{
    cli::Args,
    context::ContextInner,
    log_debug, log_fatal, log_info,
    logger::Logger,
    server::servers_setup_all,
    settings::SETTINGS_DEFAULT_LOG_MAX_BUFFER_SIZE,
    store::{Store, store_setup},
    tui::types::Tui,
};
use tokio::select;
use tokio_cron_scheduler::JobScheduler;

use gmon::logger::types::level::LogLevel;

#[tokio::main]
async fn main() {
    // Parse CLI arguments.
    let args = Args::parse();

    // Informational flags that exit before we set anything up.
    if args.list_query_types {
        Args::print_query_types();

        return;
    }

    // Parse log levels.
    let log_levels = if let Some(levels) = &args.log_levels {
        Some(Args::parse_log_levels(levels.clone()))
    } else {
        None
    };

    // Initialize logger.
    let logger = Logger::new(
        args.log_path.clone(),
        args.log_max_buffer_size
            .unwrap_or(SETTINGS_DEFAULT_LOG_MAX_BUFFER_SIZE),
        args.basic,
        log_levels,
    )
    .await;

    // We use log_internal() until we create the context.
    logger
        .log_internal(LogLevel::Info, "Starting gmon...")
        .await;

    logger
        .log_internal(LogLevel::Info, "Initialized logger...")
        .await;

    // Create empty TUI for context.
    let tui = Tui::new();

    // Initialize storage.
    logger
        .log_internal(LogLevel::Debug, "Attempting to initialize store...")
        .await;

    let store = match Store::new(&args.clone().store, args.clone().store_path) {
        Ok(store) => {
            logger
                .log_internal(LogLevel::Info, "Initialized store...")
                .await;

            store
        }
        Err(e) => {
            logger
                .log_internal(
                    LogLevel::Fatal,
                    &format!("Failed to initialize store: {}", e),
                )
                .await;

            process::exit(1);
        }
    };

    // Initialize scheduler.
    logger
        .log_internal(LogLevel::Debug, "Attempting to initialize scheduler...")
        .await;

    let sch = match JobScheduler::new().await {
        Ok(sch) => {
            logger
                .log_internal(LogLevel::Info, "Initialized scheduler...")
                .await;
            sch
        }
        Err(e) => {
            logger
                .log_internal(
                    LogLevel::Fatal,
                    &format!("Failed to initialize scheduler: {}", e),
                )
                .await;

            process::exit(1);
        }
    };

    // Create context now so that we can access crucial components like the logger and store when initializing the scheduler and TUI.
    let ctx = ContextInner::new(args, logger, tui, store, sch);
    log_debug!(ctx, "Attempting to initialize store...");

    // Attempt to initialize store.
    match store_setup(ctx.clone()).await {
        Ok(_) => {
            log_info!(ctx, "Set up store...");
        }
        Err(e) => {
            log_fatal!(ctx, "Failed to set up store: {}", e);

            process::exit(1);
        }
    }

    log_info!(ctx, "Attempting to initialize servers...");

    // Setup servers as long as we're not in isolation mode.
    match servers_setup_all(ctx.clone()).await {
        // Basic mode has nothing to print without servers, but the TUI can add them.
        Ok(0) if ctx.args.basic => {
            log_info!(ctx, "No servers to monitor, exiting...");

            return;
        }
        Ok(0) => {
            log_info!(ctx, "No servers to monitor yet, add one with 'n'...");
        }
        Ok(count) => {
            log_info!(ctx, "Set up {} server(s)...", count);
        }
        Err(e) => {
            log_fatal!(ctx, "Failed to set up servers: {}", e);

            process::exit(1);
        }
    }

    log_debug!(ctx, "Attempting to set up scheduler...");

    // Start the scheduler.
    {
        let sch = ctx.sch.read().await;

        match sch.start().await {
            Ok(_) => {
                log_info!(ctx, "Started the scheduler...");
            }
            Err(e) => {
                log_fatal!(ctx, "Failed to start scheduler: {}", e);

                process::exit(1);
            }
        }
    }

    // less code \O/
    let args = &ctx.args;

    // If we're not in basic mode, start the TUI.
    if !args.basic {
        // Prepare TUI.
        log_debug!(ctx, "Preparing TUI...");

        match ctx.tui.prepare(ctx.clone()).await {
            Ok(_) => {
                log_debug!(ctx, "Prepared TUI...");
            }
            Err(e) => {
                log_fatal!(ctx, "Failed to prepare TUI: {}", e);

                process::exit(1);
            }
        }

        // Now start the TUI.
        log_info!(ctx, "Starting TUI...");

        match Tui::start(ctx.clone()).await {
            Ok(_) => {
                log_info!(ctx, "Started TUI...");
            }
            Err(e) => {
                log_fatal!(ctx, "Failed to start TUI: {}", e);

                process::exit(1);
            }
        }
    }

    loop {
        select! {
            _ = ctx.cancel_token.cancelled() => {
                log_info!(ctx, "Cancellation signal received, shutting down...");


                break;
            }
        }
    }

    // Before anything, clean up our TUI if it was started.
    if !args.basic {
        log_info!(ctx, "Cleaning up TUI...");

        ctx.tui.cleanup().await;
    }

    // Shut down scheduler.
    {
        let mut sch = ctx.sch.write().await;

        match sch.shutdown().await {
            Ok(_) => {
                log_info!(ctx, "Shut down scheduler...");
            }
            Err(e) => {
                log_fatal!(ctx, "Failed to shut down scheduler: {}", e);

                process::exit(1);
            }
        }
    }
}
