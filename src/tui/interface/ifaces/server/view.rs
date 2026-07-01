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
    server::ServerCtx,
    tui::{
        action::TuiAction,
        interface::{context::TuiInterfaceContext, ext::TuiInterfaceExt, types::TuiInterfaceType},
    },
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

    pub fn get_server_by_id(&self, ctx: Context) -> Result<Arc<ServerCtx>> {
        let servers = match ctx.servers.try_read() {
            Ok(servers) => servers,
            Err(_) => {
                return Err(anyhow!("Failed to acquire read lock on servers"));
            }
        };

        let srv_ctx = servers
            .iter()
            .find(|s| s.id == self.server_id)
            .cloned()
            .ok_or_else(|| anyhow!("Failed to find server context"))?;

        Ok(srv_ctx)
    }
}

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceServerViewDrawData {}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceServerView> {
    type DrawData = TuiInterfaceServerViewDrawData;

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

    async fn prepare(&mut self, ctx: Context) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self, ctx: Context) -> Result<()> {
        Ok(())
    }

    fn get_key_bindings(&self) -> Vec<(String, String)> {
        vec![
            ("Esc".to_string(), "Back".to_string()),
            ("e".to_string(), "Edit".to_string()),
        ]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<TuiAction> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(TuiAction::ChangeInterface(
                TuiInterfaceType::Dashboard,
                None,
            )),
            _ => Ok(TuiAction::None),
        }
    }

    fn draw(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        ctx: Context,
        draw_data: Option<&Self::DrawData>,
    ) {
        // Retrieve server context.
        let srv_ctx = match self.interface.get_server_by_id(ctx.clone()) {
            Ok(srv_ctx) => srv_ctx,
            Err(_) => return,
        };
    }

    async fn fetch_snapshot_data(&mut self, ctx: Context) -> Result<Option<Self::DrawData>> {
        Ok(None)
    }
}
