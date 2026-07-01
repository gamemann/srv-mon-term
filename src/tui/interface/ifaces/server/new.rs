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
            context::TuiInterfaceContext, ext::TuiInterfaceExt,
            ifaces::server::view::ServerViewOpts, new::TuiInterfaceOpts, types::TuiInterfaceType,
        },
    },
};

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceServerNew {
    pub ip: String,
    pub port: u16,
    pub query_port: u16,

    pub new_server_id: Option<String>,
}

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceServerNewDrawData {}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceServerNew> {
    type DrawData = TuiInterfaceServerNewDrawData;

    fn title(&self) -> String {
        "Server Settings".to_string()
    }

    fn is_top_level(&self) -> bool {
        false
    }

    fn get_type(&self) -> TuiInterfaceType {
        TuiInterfaceType::ServerNew
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

    fn get_key_bindings(&self) -> Vec<(String, String)> {
        vec![("Esc".to_string(), "Back".to_string())]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<TuiAction> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => match &self.interface.new_server_id {
                Some(id) => Ok(TuiAction::ChangeInterface(
                    TuiInterfaceType::ServerView,
                    Some(TuiInterfaceOpts::ServerView(ServerViewOpts {
                        server_id: id.clone(),
                    })),
                )),
                None => Ok(TuiAction::None),
            },
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
    }

    async fn fetch_snapshot_data(&mut self, ctx: Context) -> Result<Option<Self::DrawData>> {
        Ok(None)
    }
}
