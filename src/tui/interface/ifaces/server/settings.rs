use anyhow::Result;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    context::Context,
    tui::{
        action::TuiAction,
        interface::{
            context::TuiInterfaceContext, ext::TuiInterfaceExt, new::TuiInterfaceOpts,
            types::TuiInterfaceType,
        },
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

    async fn prepare(&mut self, ctx: Context) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self, ctx: Context) -> Result<()> {
        Ok(())
    }

    fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        vec![("Esc", "Back")]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<TuiAction> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(TuiAction::ChangeInterface(
                TuiInterfaceType::ServerSettings,
                Some(TuiInterfaceOpts::ServerSettings(ServerSettingsOpts {
                    server_id: self.interface.server_id.clone(),
                })),
            )),
            _ => Ok(TuiAction::None),
        }
    }

    fn draw(&self, frame: &mut Frame<'_>, area: Rect, ctx: Context) {}
}
