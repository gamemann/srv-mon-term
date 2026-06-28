pub mod general;
pub mod latency;
pub mod users;
pub mod vars;

use std::sync::Arc;

use anyhow::{Result, anyhow};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    context::Context,
    server::{Server, ServerCtx},
    tui::interface::{context::TuiInterfaceContext, ext::TuiInterfaceExt, types::TuiInterfaceType},
};

#[derive(Default, Debug, Clone)]
pub struct ServerViewOpts {
    pub server_id: String,
}

impl ServerViewOpts {
    pub fn new(server_id: String) -> Self {
        Self { server_id }
    }
}

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceServerView {
    pub server_id: String,
}

impl From<ServerViewOpts> for TuiInterfaceServerView {
    fn from(opts: ServerViewOpts) -> Self {
        Self {
            server_id: opts.server_id,
        }
    }
}

impl TuiInterfaceServerView {
    pub fn new(server_id: String) -> Self {
        Self { server_id }
    }

    pub async fn get_server_by_id(&self, ctx: Context) -> Result<Arc<ServerCtx>> {
        ServerCtx::get_server_ctx_by_id(ctx.clone(), &self.server_id)
            .await
            .map_err(|_| anyhow!("Server with ID {} not found", self.server_id))
    }
}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceServerView> {
    fn title(&self) -> String {
        "Server View".to_string()
    }

    fn is_top_level(&self) -> bool {
        false
    }

    fn get_type(&self) -> TuiInterfaceType {
        TuiInterfaceType::ServerView
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        Some(TuiInterfaceType::Dashboard)
    }

    fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        vec![("Esc", "Back")]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Change back to dashboard interface.
                let tui = ctx.tui.read().await;

                match tui
                    .change_interface(TuiInterfaceType::Dashboard, None)
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(anyhow!("Failed to change interface: {}", e));
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame<'_>, area: Rect, _ctx: Context) {}
}
