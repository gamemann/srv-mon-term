use std::time::Duration;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyModifiers};
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    context::Context,
    log_error, log_trace,
    logger::Logger,
    logger::level::LogLevel,
    tui::{
        action::TuiAction,
        interface::{ext::TuiInterfaceExt, types::TuiInterfaceType},
        types::Tui,
    },
};

impl Tui {
    pub async fn setup_input(ctx: Context) -> Result<()> {
        let task_ctx = ctx.clone();

        // Unbounded so fast typing in a form never loses characters.
        let (tx, mut rx) = unbounded_channel::<Event>();

        let poll_interval = ctx.settings.read().await.tui_input_poll_interval;
        let cancel_for_poll = ctx.cancel_token.clone();

        tokio::task::spawn_blocking(move || {
            loop {
                if cancel_for_poll.is_cancelled() {
                    break;
                }

                match event::poll(Duration::from_millis(poll_interval)) {
                    Ok(true) => {
                        if let Ok(ev) = event::read()
                            && tx.send(ev).is_err()
                        {
                            break;
                        }
                    }
                    Ok(false) => {}
                    Err(_) => {}
                }
            }
        });

        tokio::spawn(async move {
            let ctx = task_ctx.clone();

            while let Some(ev) = rx.recv().await {
                let Event::Key(key) = ev else { continue };

                let (current_type, action) = {
                    let mut state = ctx.tui.state.write().await;

                    let current_type = state.interface.get_type();
                    let action = state
                        .interface
                        .handle_input(key, ctx.clone())
                        .await
                        .unwrap_or(TuiAction::None);

                    (current_type, action)
                };

                match action {
                    TuiAction::ChangeInterface(interface_type, opts) => {
                        match ctx
                            .tui
                            .change_interface(interface_type, opts, ctx.clone())
                            .await
                        {
                            Ok(_) => {
                                log_trace!(
                                    ctx,
                                    "Changed interface to {:?} due to interface action.",
                                    interface_type
                                );
                            }
                            Err(e) => {
                                log_error!(
                                    ctx,
                                    "Failed to change interface to {:?} due to interface action: {}",
                                    interface_type,
                                    e
                                );
                            }
                        }
                    }
                    TuiAction::Exit => {
                        ctx.cancel_token.cancel();

                        break;
                    }

                    TuiAction::None => {}
                }

                // Let's look for top-level/global input events/switches.
                match key.code {
                    // CTRL + C: Exit the application.
                    event::KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        ctx.cancel_token.cancel();

                        break;
                    }
                    // F1: Dashboard interface.
                    event::KeyCode::F(1) => {
                        // If we're not on Dashboard, switch to it.
                        if current_type == TuiInterfaceType::Dashboard {
                            log_trace!(
                                ctx,
                                "Already on Dashboard interface, ignoring F1 input event."
                            );
                        } else {
                            log_trace!(
                                ctx,
                                "Switching to Dashboard interface due to F1 input event."
                            );

                            match ctx
                                .tui
                                .change_interface(TuiInterfaceType::Dashboard, None, ctx.clone())
                                .await
                            {
                                Ok(_) => {
                                    log_trace!(
                                        ctx,
                                        "Switched to Dashboard interface due to F1 input event."
                                    );
                                }
                                Err(e) => {
                                    log_error!(
                                        ctx,
                                        "Failed to switch to Dashboard interface due to F1 input event: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                    // F2: Settings interface.
                    event::KeyCode::F(2) => {
                        // If we're not on Settings, switch to it.
                        if current_type == TuiInterfaceType::Settings {
                            log_trace!(
                                ctx,
                                "Already on Settings interface, ignoring F2 input event."
                            );
                        } else {
                            log_trace!(
                                ctx,
                                "Switching to Settings interface due to F2 input event."
                            );

                            match ctx
                                .tui
                                .change_interface(TuiInterfaceType::Settings, None, ctx.clone())
                                .await
                            {
                                Ok(_) => {
                                    log_trace!(
                                        ctx,
                                        "Switched to Settings interface due to F2 input event."
                                    );
                                }
                                Err(e) => {
                                    log_error!(
                                        ctx,
                                        "Failed to switch to Settings interface due to F2 input event: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                    // F3: Logs interface.
                    event::KeyCode::F(3) => {
                        // If we're not on Logs, switch to it.
                        if current_type == TuiInterfaceType::Logs {
                            log_trace!(ctx, "Already on Logs interface, ignoring F3 input event.");
                        } else {
                            log_trace!(ctx, "Switching to Logs interface due to F3 input event.");

                            // Switch to Logs interface.
                            match ctx
                                .tui
                                .change_interface(TuiInterfaceType::Logs, None, ctx.clone())
                                .await
                            {
                                Ok(_) => {
                                    log_trace!(
                                        ctx,
                                        "Switched to Logs interface due to F3 input event."
                                    );
                                }
                                Err(e) => {
                                    log_error!(
                                        ctx,
                                        "Failed to switch to Logs interface due to F3 input event: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                    // F4: About interface.
                    event::KeyCode::F(4) => {
                        // If we're not on About, switch to it.
                        if current_type == TuiInterfaceType::About {
                            log_trace!(ctx, "Already on About interface, ignoring F4 input event.");
                        } else {
                            log_trace!(ctx, "Switching to About interface due to F4 input event.");

                            // Switch to About interface.
                            match ctx
                                .tui
                                .change_interface(TuiInterfaceType::About, None, ctx.clone())
                                .await
                            {
                                Ok(_) => {
                                    log_trace!(
                                        ctx,
                                        "Switched to About interface due to F4 input event."
                                    );
                                }
                                Err(e) => {
                                    log_error!(
                                        ctx,
                                        "Failed to switch to About interface due to F4 input event: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
}
