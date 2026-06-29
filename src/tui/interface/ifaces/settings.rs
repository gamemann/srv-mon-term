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
        interface::{context::TuiInterfaceContext, ext::TuiInterfaceExt, types::TuiInterfaceType},
    },
};

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceSettings {}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceSettings> {
    fn title(&self) -> String {
        "Settings".to_string()
    }

    fn is_top_level(&self) -> bool {
        true
    }

    fn get_type(&self) -> TuiInterfaceType {
        TuiInterfaceType::Settings
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        None
    }

    async fn prepare(&mut self, ctx: Context) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self, ctx: Context) -> Result<()> {
        Ok(())
    }

    fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        vec![("Esc", "Quit")]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<TuiAction> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Ok(TuiAction::Exit),
            _ => Ok(TuiAction::None),
        }
    }

    fn draw(&self, frame: &mut Frame<'_>, area: Rect, ctx: Context) {
        // Implement the logic to draw the settings interface
    }
}
