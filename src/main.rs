#![allow(dead_code)]
#![allow(unused_variables)]

use std::process;

use clap::Parser;
use srv_mon_term::{
    cli::Args,
    context::ContextInner,
    log_debug, log_fatal, log_info,
    logger::Logger,
    server::{check_server_cli, servers_setup_all},
    store::{Store, store_setup},
    tui::types::Tui,
};
use tokio::select;
use tokio_cron_scheduler::JobScheduler;

use srv_mon_term::logger::types::level::LogLevel;

#[tokio::main]
async fn main() {
    // Parse CLI arguments.
    let args = Args::parse();

    // Parse log levels.
    let log_levels = args.parse_log_levels();

    // Initialize logger.
    let mut logger = Logger::new(log_levels, None, args.basic);

    log_info!(logger, "Starting srv-mon-term...");
    log_info!(logger, "Initialized logger...");

    // Create empty TUI for context.
    let tui = Tui::new();

    // Initialize storage.
    log_info!(logger, "Attempting to initialize store...");

    let store = match Store::new(&args.clone().store, args.clone().store_path) {
        Ok(store) => {
            log_info!(logger, "Initialized store...");

            store
        }
        Err(e) => {
            log_fatal!(logger, "Failed to initialize store: {}", e);

            process::exit(1);
        }
    };

    // Initialize scheduler.
    log_debug!(logger, "Attempting to initialize scheduler...");

    let sch = match JobScheduler::new().await {
        Ok(sch) => {
            log_info!(logger, "Initialized scheduler...");

            sch
        }
        Err(e) => {
            log_fatal!(logger, "Failed to initialize scheduler: {}", e);

            process::exit(1);
        }
    };

    // Create context now so that we can access crucial components like the logger and store when initializing the scheduler and TUI.
    let ctx = ContextInner::new(args, logger, tui, store, sch);

    // less code \O/
    let logger = &ctx.logger;
    let args = &ctx.args;

    log_debug!(logger.write().await, "Attempting to initialize store...");

    // Attempt to initialize store.
    match store_setup(ctx.clone()).await {
        Ok(_) => {
            log_info!(logger.write().await, "Set up store...");
        }
        Err(e) => {
            log_fatal!(logger.write().await, "Failed to set up store: {}", e);

            process::exit(1);
        }
    }

    log_info!(logger.write().await, "Attempting to initialize servers...");

    // Setup servers as long as we're not in isolation mode.
    match servers_setup_all(ctx.clone()).await {
        Ok(_) => {
            log_info!(logger.write().await, "Set up servers...");
        }
        Err(e) => {
            log_fatal!(logger.write().await, "Failed to set up servers: {}", e);

            process::exit(1);
        }
    }

    // Check for CLI server override.
    log_info!(logger.write().await, "Checking for server CLI overrides...");

    match check_server_cli(ctx.clone()).await {
        Ok(_) => {
            log_info!(logger.write().await, "Checked for server CLI overrides...");
        }
        Err(e) => {
            log_fatal!(
                logger.write().await,
                "Failed to check for server CLI overrides: {}",
                e
            );

            process::exit(1);
        }
    }

    log_debug!(logger.write().await, "Attempting to set up scheduler...");

    // Start the scheduler.
    {
        let sch = ctx.sch.read().await;

        match sch.start().await {
            Ok(_) => {
                log_info!(logger.write().await, "Set up scheduler...");
            }
            Err(e) => {
                log_fatal!(logger.write().await, "Failed to set up scheduler: {}", e);

                process::exit(1);
            }
        }
    }

    loop {
        select! {
            _ = ctx.cancel_token.cancelled() => {
                log_info!(logger.write().await, "Cancellation signal received, shutting down...");


                break;
            }
        }
    }

    // Shut down scheduler.
    {
        let mut sch = ctx.sch.write().await;

        match sch.shutdown().await {
            Ok(_) => {
                log_info!(logger.write().await, "Shut down scheduler...");
            }
            Err(e) => {
                log_fatal!(logger.write().await, "Failed to shut down scheduler: {}", e);

                process::exit(1);
            }
        }
    }
}
