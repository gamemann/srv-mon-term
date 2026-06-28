use anyhow::{Result, bail};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    context::Context,
    tui::interface::{
        context::TuiInterfaceContext, ext::TuiInterfaceExt, new::TuiInterfaceOpts,
        server::view::ServerViewOpts, types::TuiInterfaceType,
    },
};

#[derive(Debug, Clone)]
pub struct ServerSettingsOpts {
    pub server_id: String,
}

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceServerSettings {
    pub server_id: String,
}

impl From<ServerSettingsOpts> for TuiInterfaceServerSettings {
    fn from(opts: ServerSettingsOpts) -> Self {
        Self {
            server_id: opts.server_id,
        }
    }
}

impl TuiInterfaceServerSettings {
    pub fn new(server_id: String) -> Self {
        Self { server_id }
    }
}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceServerSettings> {
    fn title(&self) -> String {
        "Server Settings".to_string()
    }

    fn is_top_level(&self) -> bool {
        false
    }

    fn get_type(&self) -> TuiInterfaceType {
        TuiInterfaceType::ServerSettings
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        Some(TuiInterfaceType::ServerView)
    }

    fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        vec![("Esc", "Back")]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Change back to server view interface.
                let tui = ctx.tui.read().await;

                match tui
                    .change_interface(
                        TuiInterfaceType::ServerView,
                        Some(TuiInterfaceOpts::ServerView(ServerViewOpts::new(
                            self.interface.server_id.clone(),
                        ))),
                    )
                    .await
                {
                    Ok(_) => {}
                    Err(e) => bail!("Failed to change interface to ServerView: {}", e),
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame<'_>, area: Rect, ctx: Context) {}
}
