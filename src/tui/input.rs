use std::{sync::Arc, time::Duration};

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyModifiers};

use crate::{
    log_error, log_trace,
    logger::level::LogLevel,
    tui::{interface::ext::TuiInterfaceExt, types::Tui},
};

impl Tui {
    pub async fn setup_input(self: Arc<Self>) -> Result<()> {
        let ctx = self.ctx()?;

        let task_ctx = ctx.clone();

        tokio::spawn(async move {
            let ctx = task_ctx.clone();

            loop {
                match event::poll(Duration::from_millis(100)) {
                    Ok(true) => {
                        if let Event::Key(key) = event::read().unwrap() {
                            // Check for Ctrl+C to exit the application.
                            if key.modifiers.contains(KeyModifiers::CONTROL)
                                && key.code == event::KeyCode::Char('c')
                            {
                                ctx.cancel_token.cancel();

                                break;
                            }

                            // Check our state and if we have an interface set, pass our input to it.
                            {
                                let mut state = self.state.write().await;

                                if let Some(interface) = &mut state.interface {
                                    match interface.handle_input(key).await {
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
