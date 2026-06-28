use std::time::Duration;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyModifiers};

use crate::{
    context::Context,
    log_error, log_trace,
    logger::level::LogLevel,
    tui::{
        interface::{
            ext::TuiInterfaceExt,
            types::{TuiInterface, TuiInterfaceType},
        },
        types::Tui,
    },
};

impl Tui {
    pub async fn setup_input(ctx: Context) -> Result<()> {
        let task_ctx = ctx.clone();

        tokio::spawn(async move {
            let ctx = task_ctx.clone();

            loop {
                let tui = ctx.tui.read().await;
                let mut state = tui.state.write().await;

                match event::poll(Duration::from_millis(100)) {
                    Ok(true) => {
                        let ev = match event::read() {
                            Ok(ev) => ev,
                            Err(e) => {
                                log_error!(
                                    ctx.logger.write().await,
                                    "Error reading input event: {}",
                                    e
                                );

                                continue;
                            }
                        };

                        if let Event::Key(key) = ev {
                            // Let's look for top-level/global input events/switches.
                            match key.code {
                                // CTRL + C: Exit the application.
                                event::KeyCode::Char('c')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    ctx.cancel_token.cancel();

                                    break;
                                }
                                // F1: Dashboard interface.
                                event::KeyCode::F(1) => {
                                    let interface = &mut state.interface;

                                    // If we're not on Dashboard, switch to it.
                                    if interface.get_type() == TuiInterfaceType::Dashboard {
                                        log_trace!(
                                            ctx.logger.write().await,
                                            "Already on Dashboard interface, ignoring F1 input event."
                                        );
                                    } else {
                                        log_trace!(
                                            ctx.logger.write().await,
                                            "Switching to Dashboard interface due to F1 input event."
                                        );

                                        // Switch to Dashboard interface.
                                        state.interface = TuiInterface::new_interface(
                                            TuiInterfaceType::Dashboard,
                                        );
                                    }
                                }
                                // F2: Settings interface.
                                event::KeyCode::F(2) => {
                                    let interface = &mut state.interface;

                                    // If we're not on Settings, switch to it.
                                    if interface.get_type() == TuiInterfaceType::Settings {
                                        log_trace!(
                                            ctx.logger.write().await,
                                            "Already on Settings interface, ignoring F2 input event."
                                        );
                                    } else {
                                        log_trace!(
                                            ctx.logger.write().await,
                                            "Switching to Settings interface due to F2 input event."
                                        );

                                        // Switch to Settings interface.
                                        state.interface =
                                            TuiInterface::new_interface(TuiInterfaceType::Settings);
                                    }
                                }
                                // F3: Logs interface.
                                event::KeyCode::F(3) => {
                                    let interface = &mut state.interface;

                                    // If we're not on Logs, switch to it.
                                    if interface.get_type() == TuiInterfaceType::Logs {
                                        log_trace!(
                                            ctx.logger.write().await,
                                            "Already on Logs interface, ignoring F3 input event."
                                        );
                                    } else {
                                        log_trace!(
                                            ctx.logger.write().await,
                                            "Switching to Logs interface due to F3 input event."
                                        );

                                        // Switch to Logs interface.
                                        state.interface =
                                            TuiInterface::new_interface(TuiInterfaceType::Logs);
                                    }
                                }
                                _ => {}
                            }

                            // Check our state and if we have an interface set, pass our input to it.
                            {
                                let interface = &mut state.interface;

                                match interface.handle_input(key, ctx.clone()).await {
                                    Ok(_) => {
                                        log_trace!(
                                            ctx.logger.write().await,
                                            "Handled input event '{:?}' for interface: {}",
                                            key,
                                            interface.title()
                                        );
                                    }
                                    Err(e) => {
                                        log_error!(
                                            ctx.logger.write().await,
                                            "Failed to handle input event for interface '{}': {}",
                                            interface.title(),
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Ok(false) => {
                        // No input event, continue the loop.
                    }
                    Err(e) => {
                        log_error!(
                            ctx.logger.write().await,
                            "Error polling for input events: {}",
                            e
                        );
                    }
                }
            }
        });

        Ok(())
    }
}
